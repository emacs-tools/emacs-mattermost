;;; mattermost-dashboard.el --- Dashboard and channel list UI -*- lexical-binding: t; -*-

(require 'mattermost-custom)
(require 'mattermost-protocol)
(require 'mattermost-channel)

(defvar mattermost--current-server-id nil
  "Current server ID for the dashboard buffer.")

(defvar mattermost-dashboard-mode-map
  (let ((map (make-sparse-keymap)))
    (define-key map (kbd "RET") #'mattermost-dashboard-open-channel)
    (define-key map (kbd "g") #'mattermost-dashboard-refresh)
    map)
  "Keymap for `mattermost-dashboard-mode`.")

(define-derived-mode mattermost-dashboard-mode special-mode "Mattermost-Dashboard"
  "Major mode for viewing Mattermost channels list.
\\{mattermost-dashboard-mode-map}"
  (setq buffer-read-only t)
  (setq truncate-lines t))

(defun mattermost-reset-state ()
  "Reset all connection state and clear pending requests."
  (interactive)
  (when (and (boundp 'mattermost--process) mattermost--process (process-live-p mattermost--process))
    (delete-process mattermost--process))
  (setq mattermost--process nil)
  (setq mattermost--pending-requests (make-hash-table :test 'equal))
  (setq mattermost--input-buffer "")
  (message "[MM] State reset complete. Ready to reconnect."))

(defun mattermost-ensure-connected (callback)
  "Ensure connection to the bridge, prompting for missing credentials."
  (let* ((url (if (string= mattermost-server-url "")
                  (read-string "Mattermost URL (e.g., https://mattermost.local): ")
                mattermost-server-url))
         (login (if (string= mattermost-login-id "")
                    (read-string "Login ID or Email: ")
                  mattermost-login-id))
         (password (read-passwd "Password (or PAT token): "))
         (server-id mattermost-server-id))
    
    (when (string= mattermost-server-url "")
      (customize-save-variable 'mattermost-server-url url))
    (when (string= mattermost-login-id "")
      (customize-save-variable 'mattermost-login-id login))
    
    (unless (and (boundp 'mattermost--process) 
                 mattermost--process 
                 (process-live-p mattermost--process))
      (mattermost-connect-to-bridge))
    
    (message "[MM-DEBUG] Connecting to %s as %s (password masked)" url login)
    (mattermost--send-request
     "connect"
     (list :server_id server-id
           :url url
           :login_id login
           :password password)
     (lambda (response)
       (if (plist-get response :ok)
           (progn
             (message "[MM] Connected successfully.")
             (funcall callback server-id))
         (message "[MM] Connect failed: %S" (plist-get response :error)))))))

(defun mattermost-dashboard ()
  "Open the Mattermost dashboard showing available channels."
  (interactive)
  (mattermost-ensure-connected
   (lambda (server-id)
     (mattermost--send-request
      "list_channels"
      (list :server_id server-id)
      (lambda (response)
        (if (plist-get response :ok)
            (mattermost--render-dashboard 
             server-id 
             (plist-get (plist-get response :result) :channels))
          (message "[MM] Failed to list channels: %S" (plist-get response :error))))))))

(defun mattermost--render-dashboard (server-id channels)
  "Render CHANNELS in a new Mattermost Dashboard buffer."
  (let ((buf (get-buffer-create "*Mattermost Dashboard*")))
    (with-current-buffer buf
      (let ((inhibit-read-only t))
        (erase-buffer)
        (mattermost-dashboard-mode)
        (setq-local mattermost--current-server-id server-id)
        (setq-local mattermost--channels channels)
        
        (insert "Mattermost Channels\n")
        (insert "==================\n\n")
        
        (dolist (ch channels)
          (let ((name (or (plist-get ch :display_name) "Unknown"))
                (type (or (plist-get ch :channel_type) "?"))
                (id (plist-get ch :id)))
            (insert (format "[%s] %s (ID: %s)\n" type name id))))
        
        (goto-char (point-min))))
    (switch-to-buffer buf)))

(defun mattermost-dashboard-refresh ()
  "Refresh the channel list in the current dashboard buffer."
  (interactive)
  (let ((server-id mattermost--current-server-id))
    (unless server-id
      (error "No server ID associated with this buffer"))
    (message "[MM-DEBUG] Refreshing channels...")
    (mattermost--send-request
     "list_channels"
     (list :server_id server-id)
     (lambda (response)
       (if (plist-get response :ok)
           (mattermost--render-dashboard 
            server-id 
            (plist-get (plist-get response :result) :channels))
         (message "[MM] Refresh failed: %S" (plist-get response :error)))))))

(defun mattermost-dashboard-open-channel ()
  "Open the channel under point."
  (interactive)
  (let ((line (thing-at-point 'line t)))
    (if (string-match "ID: \\([a-z0-9]+\\)" line)
        (let* ((channel-id (match-string 1 line))
               (server-id mattermost--current-server-id)
               (channel-name (if (string-match "\\] \\(.+\\) (ID:" line)
                                 (match-string 1 line)
                               "Unknown")))
          (if server-id
              (mattermost-channel-open server-id channel-id channel-name)
            (error "No server ID associated with this dashboard")))
      (message "[MM] No channel ID found on this line."))))

(provide 'mattermost-dashboard)
