;;; mattermost-channel.el --- Channel buffer UI -*- lexical-binding: t; -*-

(require 'mattermost-custom)
(require 'mattermost-protocol)

(defvar mattermost-channel-mode-map
  (let ((map (make-sparse-keymap)))
    (define-key map (kbd "C-c C-s") #'mattermost-channel-send-message)
    (define-key map (kbd "g") #'mattermost-channel-refresh)
    map)
  "Keymap for `mattermost-channel-mode`.")

(define-derived-mode mattermost-channel-mode special-mode "Mattermost-Channel"
  "Major mode for viewing a Mattermost channel.
\\{mattermost-channel-mode-map}"
  (setq buffer-read-only t)
  (setq truncate-lines nil))

(defvar-local mattermost--channel-id nil
  "Channel ID for the current buffer.")

(defvar-local mattermost--server-id nil
  "Server ID for the current buffer.")

(defvar-local mattermost--channel-name nil
  "Display name of the current channel.")

(defun mattermost-channel-open (server-id channel-id channel-name)
  "Open a buffer for CHANNEL-ID on SERVER-ID with CHANNEL-NAME."
  (let ((buf-name (format "*MM: %s*" channel-name)))
    (let ((buf (get-buffer-create buf-name)))
      (with-current-buffer buf
        (mattermost-channel-mode)
        (setq-local mattermost--server-id server-id)
        (setq-local mattermost--channel-id channel-id)
        (setq-local mattermost--channel-name channel-name)
        (mattermost-channel-load-history))
      (switch-to-buffer buf))))

(defun mattermost-channel-load-history ()
  "Load and display message history for the current channel."
  (let ((server-id mattermost--server-id)
        (channel-id mattermost--channel-id)
        (channel-name mattermost--channel-name))
    (message "[MM-DEBUG] Loading history for channel %s..." channel-id)
    (mattermost--send-request
     "get_history"
     (list :server_id server-id
           :channel_id channel-id
           :limit 50)
     (lambda (response)
       (if (plist-get response :ok)
           (mattermost--render-channel-history
            channel-name
            (plist-get (plist-get response :result) :posts))
         (message "[MM] Failed to load history: %S" (plist-get response :error)))))))

(defun mattermost--format-post (post)
  "Format a POST plist for display with multi-line support and bold sender."
  (let* ((msg-text (or (plist-get post :message) ""))
         (create-at (plist-get post :create_at))
         (time-str (if create-at
                       (format-time-string "%Y-%m-%d %H:%M" (seconds-to-time (/ create-at 1000)))
                     "????-??-?? ??:??"))
         (sender (or (plist-get post :user_display_name)
                     (plist-get post :user_id)
                     "unknown")))
    (concat
     (format "[%s] **" time-str)
     (propertize sender 'face 'mattermost-sender-face)
     (format ":**\n%s\n" msg-text))))

(defun mattermost--render-channel-history (channel-name posts)
  "Render POSTS in the current channel buffer with CHANNEL-NAME."
  (let ((inhibit-read-only t))
    (erase-buffer)
    (insert (format "Channel: %s\n" channel-name))
    (insert "========================================\n\n")
    
    (dolist (post posts)
      (insert (mattermost--format-post post))
      (insert "\n")) ; Пустая строка между сообщениями
    
    (goto-char (point-max))))

(defun mattermost-channel-refresh ()
  "Reload message history for the current channel."
  (interactive)
  (message "[MM-DEBUG] Refreshing channel...")
  (mattermost-channel-load-history))

(defun mattermost-channel-send-message ()
  "Send a message to the current channel.
The message will appear automatically via WebSocket fan-out; no refresh needed."
  (interactive)
  (let* ((server-id mattermost--server-id)
         (channel-id mattermost--channel-id)
         (msg (read-string "Message: ")))
    (when (string-empty-p msg)
      (error "Message cannot be empty"))
    
    (message "[MM-DEBUG] Sending message to channel %s..." channel-id)
    (mattermost--send-request
     "send_post"
     (list :server_id server-id
           :channel_id channel-id
           :message msg)
     (lambda (response)
       (if (plist-get response :ok)
           (message "[MM] Message sent. Waiting for WS echo...")
         (message "[MM] Failed to send message: %S" (plist-get response :error)))))))

(defun mattermost-channel-on-posted (params)
  "Handle a 'posted' event. PARAMS should contain the 'post' plist."
  (let* ((post (plist-get params :post))
         (channel-id (plist-get post :channel_id)))
    (dolist (buf (buffer-list))
      (with-current-buffer buf
        (when (and (eq major-mode 'mattermost-channel-mode)
                   (string= mattermost--channel-id channel-id))
          (let ((inhibit-read-only t))
            (goto-char (point-max))
            (insert (mattermost--format-post post))
            (insert "\n") ; Пустая строка между сообщениями
            (when (= (point) (point-max))
              (goto-char (point-max)))))))))

(provide 'mattermost-channel)
