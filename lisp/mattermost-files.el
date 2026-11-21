;;; mattermost-files.el --- File attachment helpers -*- lexical-binding: t; -*-

(require 'mattermost-custom)
(require 'mattermost-protocol)

(defun mattermost--format-file-size (bytes)
  "Format BYTES as human-readable size."
  (cond
   ((< bytes 1024) (format "%d B" bytes))
   ((< bytes (* 1024 1024)) (format "%.1f KB" (/ bytes 1024.0)))
   ((< bytes (* 1024 1024 1024)) (format "%.1f MB" (/ bytes (* 1024.0 1024.0))))
   (t (format "%.1f GB" (/ bytes (* 1024.0 1024.0 1024.0))))))

(defun mattermost--format-file-block (files)
  "Return a propertized multi-line string describing FILES plist list."
  (mapconcat
   (lambda (f)
     (let* ((name (or (plist-get f :name) "unknown"))
            (size (or (plist-get f :size) 0))
            (id (plist-get f :id)))
       (propertize
        (format "  📎 %s (%s)\n" name (mattermost--format-file-size size))
        'mm-file-id id
        'mm-file-name name
        'mouse-face 'highlight
        'help-echo "RET to download and open")))
   files
   ""))

(provide 'mattermost-files)
