;;; mattermost-dm.el --- Direct Message handling -*- lexical-binding: t; -*-

(require 'mattermost-protocol)

(defvar mattermost-self-channel-id nil
  "Channel ID for the current user's self-DM.")

(defvar mattermost-self-buffer nil
  "Buffer showing the self-DM history.")

(defvar mattermost--oldest-post-id nil
  "ID of the oldest loaded post, used for pagination.")

(defvar mattermost--history-limit 10
  "Number of posts loaded per page.")

(defun mattermost-open-self-dm ()
  "Open or switch to the Self-DM buffer. Auto-connects if needed."
  (interactive)
  (mattermost--ensure-connected))

(defun mattermost--ensure-connected ()
  "Connect to bridge and Mattermost if not already connected, then open DM."
  (mattermost-connect-to-bridge)
  (if mattermost-self-channel-id
      (progn
        (mattermost-switch-to-self-dm)
        (mattermost-load-history))
    (let* ((env-pass (getenv "MM_PASSWORD"))
           (url (or (and (stringp mattermost-server-url)
                         (not (string-empty-p mattermost-server-url))
                         mattermost-server-url)
                    (read-string "Mattermost URL: ")))
           (login (or (and (stringp mattermost-login-id)
                           (not (string-empty-p mattermost-login-id))
                           mattermost-login-id)
                      (read-string "Login ID: ")))
           (password (or (and (stringp mattermost-password)
                              (not (string-empty-p mattermost-password))
                              mattermost-password)
                         env-pass
                         (read-passwd "Password: "))))
      (customize-save-variable 'mattermost-server-url url)
      (customize-save-variable 'mattermost-login-id login)
      (setq mattermost-password password)
      (message "[MM] Connecting to %s as %s..." url login)
      (mattermost--send-request
       "connect"
       (list :server_id mattermost-server-id
             :url url
             :login_id login
             :password password)
       #'mattermost--on-connect-response))))

(defun mattermost--on-connect-response (resp)
  "Handle response from connect request."
  (if (equal (plist-get resp :ok) t)
      (progn
        (message "[MM] Connected successfully!")
        (mattermost-init-self-dm))
    (let ((err (plist-get resp :error)))
      (message "[MM] Connection FAILED: %s"
               (if (listp err) (mm--extract err "message") err)))))

(defun mattermost-init-self-dm ()
  "Request Self-DM channel from bridge."
  (mattermost--send-request
   "open_self_dm"
   (list :server_id mattermost-server-id)
   #'mattermost--on-self-dm-response))

(defun mattermost--on-self-dm-response (resp)
  "Handle response from open_self_dm request."
  (if (equal (plist-get resp :ok) t)
      (let* ((result (plist-get resp :result))
             (channel (plist-get result :channel))
             (id (plist-get channel :id)))
        (setq mattermost-self-channel-id id)
        (mattermost-switch-to-self-dm)
        (mattermost-load-history))
    (message "[MM] Failed to open Self-DM: %s" (plist-get resp :error))))

(defun mattermost-switch-to-self-dm ()
  "Switch to the Self-DM buffer."
  (let ((buf-name "*Mattermost[work]/DM:self*"))
    (setq mattermost-self-buffer (get-buffer-create buf-name))
    (switch-to-buffer mattermost-self-buffer)
    (mattermost-mode)))

(defun mattermost-load-history ()
  "Load the latest page of self-DM history."
  (interactive)
  (unless (and mattermost-self-channel-id
               (not (string-empty-p mattermost-self-channel-id)))
    (user-error "No self-DM channel yet"))
  (setq mattermost--oldest-post-id nil)
  (mattermost--send-request
   "get_history"
   (list :server_id mattermost-server-id
         :channel_id mattermost-self-channel-id
         :limit mattermost--history-limit)
   #'mattermost--on-history-response))

(defun mattermost-load-earlier ()
  "Load the previous page of history (older messages)."
  (interactive)
  (unless mattermost--oldest-post-id
    (user-error "Nothing loaded yet, or no earlier messages"))
  (mattermost--send-request
   "get_history"
   (list :server_id mattermost-server-id
         :channel_id mattermost-self-channel-id
         :limit mattermost--history-limit
         :before mattermost--oldest-post-id)
   #'mattermost--on-earlier-response))

(defun mattermost--on-history-response (resp)
  "Handle initial get_history response."
  (if (equal (plist-get resp :ok) t)
      (let ((posts (mm--extract (plist-get resp :result) "posts")))
        (mattermost--render-history posts t))
    (message "[MM] Failed to load history: %s" (plist-get resp :error))))

(defun mattermost--on-earlier-response (resp)
  "Handle paginated get_history response."
  (if (equal (plist-get resp :ok) t)
      (let ((posts (mm--extract (plist-get resp :result) "posts")))
        (if (and posts (> (length posts) 0))
            (mattermost--render-history posts nil)
          (message "[MM] No earlier messages")))
    (message "[MM] Failed to load earlier: %s" (plist-get resp :error))))

(defun mattermost--render-history (posts initial)
  "Render POSTS into the self-DM buffer.
If INITIAL, clear the buffer and render the whole page.
Otherwise prepend older posts under the header."
  (unless (buffer-live-p mattermost-self-buffer)
    (error "Self-DM buffer is gone"))
  (with-current-buffer mattermost-self-buffer
    (let ((inhibit-read-only t))
      (if initial
          (progn
            (erase-buffer)
            (mattermost--insert-header)
            (dolist (post posts)
              (mattermost--insert-post post))
            ;; oldest = first post of the ascending page
            (when posts
              (setq mattermost--oldest-post-id
                    (mm--extract (car posts) "id"))))
        ;; prepend: go to end of header line
        (save-excursion
          (goto-char (point-min))
          (forward-line 1)
          (let ((start (point)))
            (dolist (post posts)
              (mattermost--insert-post post))
            (when posts
              (setq mattermost--oldest-post-id
                    (mm--extract (car posts) "id")))))
        (message "[MM] Loaded %d earlier messages" (length posts))))))

(defun mattermost--insert-header ()
  "Insert buffer header."
  (insert (format "*Mattermost[work]/DM:self* — %d latest messages\n"
                  mattermost--history-limit))
  (insert "Press 'l' to load earlier messages, C-c C-c to send.\n\n"))

(defun mattermost--insert-post (post)
  "Insert a single POST at point."
  (let ((msg (mm--extract post "message"))
        (user-id (mm--extract post "user_id"))
        (files (mm--extract post "file_ids"))
        (ts (mm--extract post "create_at")))
    (insert (format "[%s] %s%s\n"
                    (mattermost--format-time ts)
                    msg
                    (if (and (listp files) (> (length files) 0))
                        (format "  [+%d file(s)]" (length files))
                      "")))))

(defun mattermost--format-time (ms)
  "Format epoch milliseconds MS as HH:MM."
  (if (and (numberp ms) (> ms 0))
      (format-time-string "%H:%M" (seconds-to-time (/ ms 1000)))
    "??:??"))

;; ============================================================
;; УНИВЕРСАЛЬНАЯ ФУНКЦИЯ ИЗВЛЕЧЕНИЯ ДАННЫХ
;; ============================================================

(defun mm--extract (obj key)
  "Safely extract KEY from OBJ, handling plists, alists (string/symbol), and hash tables."
  (let ((key-sym (if (stringp key) (intern key) key))
        (key-str (if (stringp key) key (symbol-name key)))
        (key-kw (if (stringp key) (intern (concat ":" key)) (intern (concat ":" (symbol-name key))))))
    (or (plist-get obj key-kw)
        (plist-get obj key-sym)
        (cdr (assq key-sym obj))
        (cdr (assoc key-str obj))
        (and (hash-table-p obj) (gethash key-str obj))
        (and (hash-table-p obj) (gethash key-sym obj))
        "[NOT FOUND]")))

;; ============================================================

(define-derived-mode mattermost-mode text-mode "Mattermost"
  "Major mode for Mattermost chat buffers."
  (setq-local comment-start "# ")
  (setq buffer-read-only nil)
  (local-set-key (kbd "C-c C-c") #'mattermost-send-message)
  (local-set-key (kbd "l") #'mattermost-load-earlier))

(defun mattermost-send-message ()
  "Send the current line as a message."
  (interactive)
  (unless mattermost-self-channel-id
    (user-error "Not connected"))
  (let ((msg (buffer-substring (line-beginning-position) (line-end-position))))
    (mattermost--send-request
     "send_post"
     (list :server_id mattermost-server-id
           :channel_id mattermost-self-channel-id
           :message msg)
     #'mattermost--on-send-response)))

(defun mattermost--on-send-response (_resp)
  "Handle response after sending a message."
  (with-current-buffer (or mattermost-self-buffer (current-buffer))
    (let ((inhibit-read-only t))
      (delete-region (line-beginning-position) (line-end-position))
      (insert "\n"))))

(provide 'mattermost-dm)
