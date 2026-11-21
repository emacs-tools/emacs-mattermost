;;; mattermost-dashboard.el --- Dashboard and channel list UI -*- lexical-binding: t; -*-

(require 'mattermost-custom)
(require 'mattermost-protocol)
(require 'mattermost-channel)

(defvar mattermost--current-server-id nil
  "Server id owning the current dashboard buffer.")
(defvar mattermost--channels nil
  "Last channel list rendered in this buffer.")
(defvar mattermost--teams nil
  "Last team list rendered in this buffer.")
(defvar mattermost--collapsed-teams nil
  "List of team IDs currently collapsed in the dashboard.")

(defvar mattermost-dashboard-mode-map
  (let ((map (make-sparse-keymap)))
    (define-key map (kbd "RET") #'mattermost-dashboard-open-channel)
    (define-key map (kbd "TAB") #'mattermost-dashboard-toggle-section)
    (define-key map (kbd "<backtab>") #'mattermost-dashboard-toggle-all)
    (define-key map (kbd "g") #'mattermost-dashboard-refresh)
    map)
  "Keymap for `mattermost-dashboard-mode'.")

(define-derived-mode mattermost-dashboard-mode special-mode "Mattermost"
  "Major mode for the Mattermost channel list.
\\{mattermost-dashboard-mode-map}
TAB on a team header collapses/expands its section.
S-TAB (backtab) collapses/expands all sections."
  (setq buffer-read-only t)
  (setq truncate-lines t)
  (add-to-invisibility-spec 'mm-section))

(defun mattermost-reset-state ()
  "Reset bridge connection state and pending requests."
  (interactive)
  (when (and mattermost--process (process-live-p mattermost--process))
    (delete-process mattermost--process))
  (setq mattermost--process nil)
  (setq mattermost--pending-requests (make-hash-table :test 'equal))
  (setq mattermost--input-buffer "")
  (setq mattermost--collapsed-teams nil)
  (when (get-buffer "*Mattermost Dashboard*")
    (kill-buffer "*Mattermost Dashboard*"))
  (message "[MM] State reset complete."))

(defun mattermost-ensure-connected (callback)
  "Connect to bridge and Mattermost, then call CALLBACK with server-id."
  (let* ((url (if (string-empty-p mattermost-server-url)
                  (read-string "Mattermost URL: ")
                mattermost-server-url))
         (login (if (string-empty-p mattermost-login-id)
                    (read-string "Login ID or email: ")
                  mattermost-login-id))
         (password (read-passwd "Password: "))
         (server-id mattermost-server-id))
    (when (string-empty-p mattermost-server-url)
      (customize-save-variable 'mattermost-server-url url))
    (when (string-empty-p mattermost-login-id)
      (customize-save-variable 'mattermost-login-id login))
    (unless (and mattermost--process (process-live-p mattermost--process))
      (mattermost-connect-to-bridge))
    (message "[MM-DEBUG] Connecting to %s as %s (password ***MASKED***)" url login)
    (mattermost--send-request
     "connect"
     (list :server_id server-id :url url :login_id login :password password)
     (lambda (response)
       (if (plist-get response :ok)
           (progn
             (message "[MM] Connected.")
             (funcall callback server-id))
         (message "[MM] Connect failed: %S" (plist-get response :error)))))))

(defun mattermost-dashboard ()
  "Open the Mattermost dashboard."
  (interactive)
  (mattermost-ensure-connected #'mattermost--request-channels))

(defun mattermost--request-channels (server-id)
  "Fetch channels and teams for SERVER-ID, then render the dashboard."
  (message "[MM-DEBUG] Requesting list_channels for %s" server-id)
  (mattermost--send-request
   "list_channels"
   (list :server_id server-id)
   (lambda (response)
     (if (plist-get response :ok)
         (let ((result (plist-get response :result)))
           (mattermost--render-dashboard
            server-id
            (plist-get result :channels)
            (plist-get result :teams)))
       (message "[MM] list_channels failed: %S" (plist-get response :error))))))

(defun mattermost--insert-team-header (title)
  "Insert a section header TITLE with collapse marker.  Return (start . end)."
  (let ((start (point)))
    (insert (format "%s ▼\n" title))
    (put-text-property start (point) 'face 'mattermost-sender-face)
    (cons start (point))))

(defun mattermost--insert-channel-line (ch)
  "Insert one selectable line for channel CH."
  (let* ((id (plist-get ch :id))
         (type (or (plist-get ch :type) "?"))
         (name (or (plist-get ch :display_name) ""))
         (label (if (string-empty-p name) (or (plist-get ch :name) id) name))
         (line (format "  [%s] %s\n" type label)))
    (insert (propertize line
                        'mm-channel-id id
                        'mm-channel-name label
                        'mouse-face 'highlight))))

(defun mattermost--sort-channels-by-activity (channels)
  "Sort CHANNELS by last_post_at descending."
  (sort channels
        (lambda (a b)
          (> (or (plist-get a :last_post_at) 0)
             (or (plist-get b :last_post_at) 0)))))

(defun mattermost--set-header-marker (start end marker)
  "Replace the collapse marker between START and END with MARKER.
Caller must bind `inhibit-read-only'."
  (save-excursion
    (goto-char start)
    (when (re-search-forward "[▼▶]" end t)
      (replace-match marker t t))))

(defun mattermost--collapse-section (overlay collapse)
  "Set COLLAPSE state on section OVERLAY and update its header marker."
  (let ((inhibit-read-only t)
        (team-id (overlay-get overlay 'mm-team-id)))
    (overlay-put overlay 'invisible (and collapse 'mm-section))
    (mattermost--set-header-marker
     (overlay-get overlay 'mm-header-start)
     (overlay-get overlay 'mm-header-end)
     (if collapse "▶" "▼"))
    (if collapse
        (unless (member team-id mattermost--collapsed-teams)
          (setq mattermost--collapsed-teams
                (cons team-id mattermost--collapsed-teams)))
      (setq mattermost--collapsed-teams
            (delete team-id mattermost--collapsed-teams)))))

(defun mattermost-dashboard-toggle-section ()
  "Toggle visibility of the section whose header is at point."
  (interactive)
  (let ((overlay (get-text-property (point) 'mm-section-overlay)))
    (if (not overlay)
        (message "[MM] Not on a team header.")
      (mattermost--collapse-section overlay (not (overlay-get overlay 'invisible)))
      (message "[MM-DEBUG] collapsed-teams=%S" mattermost--collapsed-teams))))

(defun mattermost-dashboard-toggle-all ()
  "Collapse or expand all team sections."
  (interactive)
  (let ((overlays nil)
        (any-expanded nil))
    (save-excursion
      (goto-char (point-min))
      (while (not (eobp))
        (let ((ov (get-text-property (point) 'mm-section-overlay)))
          (when (and ov (not (memq ov overlays)))
            (push ov overlays)
            (unless (overlay-get ov 'invisible)
              (setq any-expanded t))))
        (forward-line 1)))
    (dolist (ov overlays)
      (mattermost--collapse-section ov any-expanded))
    (message "[MM-DEBUG] collapsed-teams=%S" mattermost--collapsed-teams)))

(defun mattermost--insert-section (title team-id channels)
  "Insert section TITLE for TEAM-ID with CHANNELS, honouring collapse state."
  (let* ((header (mattermost--insert-team-header title))
         (section-start (point)))
    (dolist (ch channels)
      (mattermost--insert-channel-line ch))
    (let ((ov (make-overlay section-start (point))))
      (overlay-put ov 'mm-header-start (car header))
      (overlay-put ov 'mm-header-end (cdr header))
      (overlay-put ov 'mm-team-id team-id)
      (put-text-property (car header) (cdr header) 'mm-section-overlay ov)
      (when (member team-id mattermost--collapsed-teams)
        (overlay-put ov 'invisible 'mm-section)
        (mattermost--set-header-marker (car header) (cdr header) "▶")))))

(defun mattermost--render-dashboard (server-id channels teams)
  "Render CHANNELS grouped by TEAMS for SERVER-ID, preserving collapse state."
  (message "[MM-DEBUG] Rendering: collapsed-teams=%S" mattermost--collapsed-teams)
  (let ((by-team (make-hash-table :test 'equal))
        (dms nil)
        (buf (get-buffer-create "*Mattermost Dashboard*")))
    (setq channels (mattermost--sort-channels-by-activity channels))
    (dolist (ch channels)
      (if (member (plist-get ch :type) '("D" "G"))
          (push ch dms)
        (let* ((tid (or (plist-get ch :team_id) ""))
               (prev (gethash tid by-team)))
          (puthash tid (cons ch prev) by-team))))
    (with-current-buffer buf
      (let ((inhibit-read-only t))
        (mattermost-dashboard-mode)
        (erase-buffer)
        (setq-local mattermost--current-server-id server-id)
        (setq-local mattermost--channels channels)
        (setq-local mattermost--teams teams)
        (insert "Mattermost channels    RET open · TAB toggle · g refresh\n\n")
        (let ((rendered (make-hash-table :test 'equal)))
          (dolist (team teams)
            (let* ((tid (plist-get team :id))
                   (chs (gethash tid by-team)))
              (when chs
                (puthash tid t rendered)
                (mattermost--insert-section
                 (plist-get team :display_name) tid (nreverse chs))
                (insert "\n"))))
          (maphash
           (lambda (tid chs)
             (unless (gethash tid rendered)
               (mattermost--insert-section
                (if (string-empty-p tid) "Other" tid) tid (nreverse chs))
               (insert "\n")))
           by-team))
        (when dms
          (mattermost--insert-section "Direct messages" "dms" (nreverse dms))
          (insert "\n"))
        (goto-char (point-min))))
    (switch-to-buffer buf)))

(defun mattermost-dashboard-refresh ()
  "Refresh the channel list, preserving collapse state."
  (interactive)
  (unless mattermost--current-server-id
    (error "No server ID associated with this buffer"))
  (mattermost--request-channels mattermost--current-server-id))

(defun mattermost-dashboard-open-channel ()
  "Open the channel under point."
  (interactive)
  (let ((id (get-text-property (point) 'mm-channel-id))
        (name (get-text-property (point) 'mm-channel-name)))
    (cond
     ((not id) (message "[MM] No channel on this line."))
     ((not mattermost--current-server-id)
      (error "No server ID associated with this dashboard"))
     (t (mattermost-channel-open mattermost--current-server-id id name)))))

(provide 'mattermost-dashboard)
