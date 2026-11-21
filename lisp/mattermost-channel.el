;;; mattermost-channel.el --- Channel buffer UI -*- lexical-binding: t; -*-

(require 'mattermost-custom)
(require 'mattermost-protocol)
(require 'mattermost-files)

(defvar mattermost-channel-mode-map
  (let ((map (make-sparse-keymap)))
    ;; Отцепляемся от special-mode-map, чтобы его одиночные клавиши
    ;; (space, DEL, g, q, h, ...) не мешали набору текста в черновике.
    (set-keymap-parent map nil)
    (define-key map (kbd "RET") #'mattermost-channel-ret)
    (define-key map (kbd "SPC") #'mattermost-channel-space)
    (define-key map (kbd "DEL") #'mattermost-channel-backspace)
    (define-key map "g" #'mattermost-channel-g)
    (define-key map "q" #'mattermost-channel-q)
    (define-key map "h" #'mattermost-channel-h)
    (define-key map "?" #'mattermost-channel-h)
    (define-key map (kbd "C-c C-c") #'mattermost-channel-send-message)
    (define-key map (kbd "C-c C-s") #'mattermost-channel-send-message)
    (define-key map [remap self-insert-command] #'mattermost-channel-self-insert)
    map)
  "Keymap for `mattermost-channel-mode'.
Single-key commands are context wrappers: they type in the draft and
act on history otherwise.")

(define-derived-mode mattermost-channel-mode special-mode "Mattermost-Channel"
  "Major mode for viewing and writing in a Mattermost channel.
History above the \"> \" prompt is protected by a local before-change
hook; the draft after the prompt is editable and kept at buffer end.
\\{mattermost-channel-mode-map}"
  (setq buffer-read-only nil)
  (setq truncate-lines nil)
  ;; Построчная прокрутка стрелками, без перекидывания экрана
  (setq-local scroll-conservatively 101)
  (setq-local scroll-margin 0)
  (add-hook 'before-change-functions #'mattermost--protect-history nil t))

(defvar-local mattermost--channel-id nil
  "Channel ID for the current buffer.")

(defvar-local mattermost--server-id nil
  "Server ID for the current buffer.")

(defvar-local mattermost--channel-name nil
  "Display name of the current channel.")

(defvar-local mattermost--posts nil
  "List of post plists currently displayed, oldest first.")

(defvar-local mattermost--input-text ""
  "Draft message text kept across re-renders.")

(defvar-local mattermost--input-start nil
  "Buffer position where the editable draft begins.")

(defvar-local mattermost--rendering nil
  "Non-nil while the buffer is re-rendered programmatically.")

(defvar-local mattermost--sending nil
  "Non-nil while a send_post request is in flight.")

(defun mattermost--protect-history (beg _end)
  "Forbid modifications starting in the history area (before the draft)."
  (unless mattermost--rendering
    (when (and mattermost--input-start (< beg mattermost--input-start))
      (user-error "History is read-only"))))

(defun mattermost--in-draft-p ()
  "Return non-nil when point is inside the editable draft."
  (and mattermost--input-start (>= (point) mattermost--input-start)))

(defun mattermost--channel-window ()
  "Return a window displaying the current buffer, searching all frames."
  (get-buffer-window (current-buffer) t))

(defun mattermost-channel-open (server-id channel-id channel-name)
  "Open a buffer for CHANNEL-ID on SERVER-ID with CHANNEL-NAME."
  (let ((buf-name (format "*MM: %s*" channel-name)))
    (let ((buf (get-buffer-create buf-name)))
      (with-current-buffer buf
        (mattermost-channel-mode)
        (setq-local mattermost--server-id server-id)
        (setq-local mattermost--channel-id channel-id)
        (setq-local mattermost--channel-name channel-name)
        (setq-local mattermost--posts nil)
        (setq-local mattermost--input-text "")
        (setq-local mattermost--input-start nil)
        (mattermost-channel-load-history))
      (switch-to-buffer buf)
      (with-current-buffer buf
        (goto-char (point-max))
        (when-let ((win (mattermost--channel-window)))
          (set-window-point win (point-max))
          (with-selected-window win
            (recenter -1))))
      buf)))

(defun mattermost-channel-load-history ()
  "Load message history for the current channel into `mattermost--posts'."
  (let ((server-id mattermost--server-id)
        (channel-id mattermost--channel-id)
        (buf (current-buffer)))
    (message "[MM-DEBUG] Loading history for channel %s..." channel-id)
    (mattermost--send-request
     "get_history"
     (list :server_id server-id
           :channel_id channel-id
           :limit 50)
     (lambda (response)
       (when (buffer-live-p buf)
         (with-current-buffer buf
           (if (plist-get response :ok)
               (progn
                 (setq mattermost--posts
                       (plist-get (plist-get response :result) :posts))
                 (mattermost--render-channel-buffer))
             (message "[MM] Failed to load history: %S"
                      (plist-get response :error)))))))))

(defun mattermost--format-post (post)
  "Format a POST plist: timestamp, bold sender, message, file block."
  (let* ((msg-text (or (plist-get post :message) ""))
         (create-at (plist-get post :create_at))
         (time-str (if create-at
                       (format-time-string "%Y-%m-%d %H:%M"
                                           (seconds-to-time (/ create-at 1000)))
                     "????-??-?? ??:??"))
         (sender (or (plist-get post :user_display_name)
                     (plist-get post :user_id)
                     "unknown"))
         (files (plist-get post :files))
         (has-content (or (not (string-empty-p msg-text)) files)))
    (unless has-content
      (setq msg-text "[empty message]"))
    (concat
     (format "[%s] **" time-str)
     (propertize sender 'face 'mattermost-sender-face)
     (format ":**\n%s"
             (if (string-empty-p msg-text) "" (concat msg-text "\n")))
     (when files (mattermost--format-file-block files))
     "\n")))

(defun mattermost--render-channel-buffer (&optional new-draft)
  "Re-render history, prompt and draft.
When NEW-DRAFT is non-nil, replace the draft with it instead of
recapturing from the buffer.  If the user was at the bottom (in the
draft or with the buffer end visible), point returns to the draft and
the window is recentered to show the prompt; otherwise the window view
is left undisturbed."
  (let* ((mattermost--rendering t)
         (win (mattermost--channel-window))
         (old-point (point))
         (old-window-start (when win (window-start win)))
         (was-in-draft (and mattermost--input-start
                            (>= old-point mattermost--input-start)))
         (was-at-bottom (or (null win)
                            (pos-visible-in-window-p (point-max) win)))
         (draft-offset nil))
    (when new-draft
      (setq mattermost--input-text new-draft))
    (when (and (null new-draft)
               mattermost--input-start
               (>= (point-max) mattermost--input-start))
      (setq mattermost--input-text
            (buffer-substring-no-properties mattermost--input-start (point-max)))
      (when was-in-draft
        (setq draft-offset (- old-point mattermost--input-start))))
    (erase-buffer)
    (insert (format "Channel: %s\n" mattermost--channel-name))
    (insert "========================================\n\n")
    (dolist (post mattermost--posts)
      (insert (mattermost--format-post post))
      (insert "\n"))
    (insert (propertize "> " 'face 'mattermost-sender-face))
    (setq mattermost--input-start (point))
    (insert mattermost--input-text)
    (cond
     ((or was-in-draft was-at-bottom)
      (goto-char (if draft-offset
                     (min (+ mattermost--input-start draft-offset) (point-max))
                   (point-max)))
      (when win
        (set-window-point win (point))
        (with-selected-window win
          (recenter -1))))
     (t
      (goto-char (min old-point (max 1 (1- mattermost--input-start))))
      (when win
        (set-window-start win old-window-start t)
        (set-window-point win (point)))))))

(defun mattermost-channel-refresh ()
  "Reload message history, keeping the current draft."
  (interactive)
  (message "[MM-DEBUG] Refreshing channel...")
  (mattermost-channel-load-history))

(defun mattermost-channel-self-insert ()
  "Jump to the draft before inserting a character typed in history."
  (interactive)
  (unless (mattermost--in-draft-p)
    (goto-char (point-max)))
  (self-insert-command 1))

(defun mattermost-channel-space (arg)
  "Insert a space in the draft; scroll down when in history."
  (interactive "p")
  (if (mattermost--in-draft-p)
      (self-insert-command arg)
    (scroll-up-command arg)))

(defun mattermost-channel-backspace (arg)
  "Delete backwards in the draft; scroll up when in history."
  (interactive "p")
  (if (mattermost--in-draft-p)
      (delete-backward-char arg)
    (scroll-down-command arg)))

(defun mattermost-channel-g ()
  "Insert g in the draft; refresh history otherwise."
  (interactive)
  (if (mattermost--in-draft-p)
      (self-insert-command 1)
    (mattermost-channel-refresh)))

(defun mattermost-channel-q ()
  "Insert q in the draft; quit window otherwise."
  (interactive)
  (if (mattermost--in-draft-p)
      (self-insert-command 1)
    (quit-window)))

(defun mattermost-channel-h ()
  "Insert the typed key in the draft; describe mode otherwise."
  (interactive)
  (if (mattermost--in-draft-p)
      (self-insert-command 1)
    (describe-mode)))

(defun mattermost-channel-ret ()
  "In the draft send the message; in history open the file link."
  (interactive)
  (if (mattermost--in-draft-p)
      (mattermost-channel-send-message)
    (mattermost-channel-open-link)))

(defun mattermost-channel-send-message ()
  "Send the draft text to the current channel and clear the draft."
  (interactive)
  (when mattermost--sending
    (user-error "Message send already in progress"))
  (let* ((server-id mattermost--server-id)
         (channel-id mattermost--channel-id)
         (buf (current-buffer))
         (msg (if (and mattermost--input-start
                       (>= (point-max) mattermost--input-start))
                  (buffer-substring-no-properties
                   mattermost--input-start (point-max))
                mattermost--input-text)))
    (when (string-empty-p (string-trim msg))
      (user-error "Message is empty"))
    (setq mattermost--sending t)
    (mattermost--render-channel-buffer "")
    (mattermost--send-request
     "send_post"
     (list :server_id server-id
           :channel_id channel-id
           :message msg)
     (lambda (response)
       (when (buffer-live-p buf)
         (with-current-buffer buf
           (setq mattermost--sending nil)
           (unless (plist-get response :ok)
             (mattermost--render-channel-buffer msg)
             (message "[MM] Failed to send message: %S"
                      (plist-get response :error)))))))))

(defun mattermost-channel-open-link ()
  "Download the file attachment under point and open it with xdg-open."
  (interactive)
  (let ((file-id (get-text-property (point) 'mm-file-id))
        (file-name (get-text-property (point) 'mm-file-name)))
    (cond
     ((and file-id file-name)
      (let ((server-id mattermost--server-id)
            (download-dir (expand-file-name mattermost-download-dir)))
        (unless server-id
          (error "No server ID associated with this buffer"))
        (message "[MM] Downloading file: %s to %s..." file-name download-dir)
        (mattermost--send-request
         "download_file"
         (list :server_id server-id
               :file_id file-id
               :file_name file-name
               :download_dir download-dir)
         (lambda (response)
           (if (plist-get response :ok)
               (let* ((result (plist-get response :result))
                      (local-path (plist-get result :local_path)))
                 (message "[MM] File saved to: %s" local-path)
                 (start-process "xdg-open" nil "xdg-open" local-path))
             (message "[MM] Failed to download file: %S"
                      (plist-get response :error)))))))
     (t
      (message "[MM] No downloadable link under point.")))))

(defun mattermost-channel-on-posted (params)
  "Handle a 'posted' event: append the post and re-render, keeping the draft."
  (let* ((post (plist-get params :post))
         (channel-id (plist-get post :channel_id)))
    (dolist (buf (buffer-list))
      (with-current-buffer buf
        (when (and (eq major-mode 'mattermost-channel-mode)
                   (string= mattermost--channel-id channel-id))
          (setq mattermost--posts (append mattermost--posts (list post)))
          (mattermost--render-channel-buffer))))))

(provide 'mattermost-channel)
