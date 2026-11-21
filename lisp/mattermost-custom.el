;;; mattermost-custom.el --- Customization options for mattermost.el -*- lexical-binding: t; -*-

(defgroup mattermost nil
  "Mattermost client for Emacs."
  :group 'applications)

(defcustom mattermost-bridge-host "127.0.0.1"
  "Host where the Mattermost bridge is listening."
  :type 'string
  :group 'mattermost)

(defcustom mattermost-bridge-port 8765
  "Port where the Mattermost bridge is listening."
  :type 'integer
  :group 'mattermost)

(defcustom mattermost-thread-display 'separate-buffer
  "How to display threads.
If `separate-buffer', open a new buffer for each thread.
If `inline', show replies in the main buffer."
  :type '(choice (const separate-buffer)
                 (const inline))
  :group 'mattermost)

(defcustom mattermost-server-url ""
  "URL of your Mattermost server (e.g. https://mattermost.local)."
  :type 'string
  :group 'mattermost)

(defcustom mattermost-login-id ""
  "Your Mattermost login ID or email."
  :type 'string
  :group 'mattermost)

(defcustom mattermost-password ""
  "Your Mattermost password. Consider using auth-source instead for security."
  :type 'string
  :group 'mattermost)

(defcustom mattermost-server-id "work"
  "Internal server identifier used for token storage."
  :type 'string
  :group 'mattermost)

(defface mattermost-sender-face
  '((t (:weight bold)))
  "Face used for message sender names in channel buffers."
  :group 'mattermost)

(provide 'mattermost-custom)
