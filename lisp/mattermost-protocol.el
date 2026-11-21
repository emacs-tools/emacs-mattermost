;;; mattermost-protocol.el --- TCP JSONL protocol handler -*- lexical-binding: t; -*-

(require 'json)
(require 'mattermost-custom)

(defvar mattermost--process nil
  "The process object for the bridge connection.")

(defvar mattermost--pending-requests (make-hash-table :test 'equal)
  "Hash table of pending requests indexed by ID.")

(defvar mattermost--input-buffer ""
  "Accumulator for incomplete TCP chunks.")

(defun mattermost-connect-to-bridge ()
  "Connect to the running Rust bridge."
  (interactive)
  (when (and (boundp 'mattermost--process) mattermost--process (process-live-p mattermost--process))
    (delete-process mattermost--process))
  
  (setq mattermost--process
        (open-network-stream
         "mattermost-bridge"
         "*mattermost-log*"
         mattermost-bridge-host
         mattermost-bridge-port))
  
  (setq mattermost--input-buffer "")
  (set-process-filter mattermost--process #'mattermost--process-filter)
  (message "[MM-PROTOCOL] Connected to Mattermost bridge."))

(defun mattermost--process-filter (_proc output)
  "Handle OUTPUT from the bridge process with proper chunk buffering."
  (setq mattermost--input-buffer (concat mattermost--input-buffer output))
  
  (let ((lines (split-string mattermost--input-buffer "\n" t)))
    (if (string-suffix-p "\n" output)
        (setq mattermost--input-buffer "")
      (setq mattermost--input-buffer (car (last lines)))
      (setq lines (butlast lines)))
    
    (dolist (line lines)
      (let ((msg (ignore-errors (json-parse-string line :object-type 'plist :array-type 'list))))
        (if msg
            (let ((id (plist-get msg :id))
                  (type (plist-get msg :type)))
              (cond
               ((string= type "response")
                (let ((cb (gethash id mattermost--pending-requests)))
                  (if cb
                      (progn
                        (funcall cb msg)
                        (remhash id mattermost--pending-requests))
                    (message "[MM-PROTOCOL] WARNING: No callback for id=%s" id))))
               ((string= type "event")
                (mattermost--handle-event msg))))
          (message "[MM-PROTOCOL] Failed to parse JSON: %S" 
                   (substring line 0 (min 100 (length line)))))))))

(defun mattermost--send-request (method params callback)
  "Send a request with METHOD and PARAMS, calling CALLBACK on response."
  (let* ((id (format "req-%d" (random 100000)))
         (payload (list :id id :type "request" :method method :params params))
         (json-str (concat (json-encode payload) "\n"))
         (safe-params (copy-tree params)))
    
    (when (plist-get safe-params :password)
      (plist-put safe-params :password "***MASKED***"))
    
    (let ((safe-payload (list :id id :type "request" :method method :params safe-params)))
      (message "[MM-PROTOCOL] Sending request id=%s method=%s" id method))
    
    (puthash id callback mattermost--pending-requests)
    (process-send-string mattermost--process json-str)))

(defun mattermost--handle-event (msg)
  "Handle an asynchronous event MSG from the bridge."
  (let ((method (plist-get msg :method))
        (params (plist-get msg :params)))
    (cond
     ((string= method "posted")
      (mattermost-channel-on-posted params))
     (t
      (message "[MM-DEBUG] Unhandled event method: %s" method)))))

(provide 'mattermost-protocol)
