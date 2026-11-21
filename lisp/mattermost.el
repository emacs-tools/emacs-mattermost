;;; mattermost.el --- Emacs bridge for Mattermost -*- lexical-binding: t; -*-

;; Copyright (C) 2026 Ivan I. Ovchinnikov
;; Author: Ivan I. Ovchinnikov <mail@iovchinnikov.ru>
;; Version: 0.1.0
;; Package-Requires: ((emacs "28.1"))
;; Keywords: comm, tools
;; URL: https://git.iovchinnikov.ru/ivan-igorevich/emacs-mattermost

;; This program is free software: you can redistribute it and/or modify
;; it under the terms of the GNU General Public License as published by
;; the Free Software Foundation, either version 3 of the License, or
;; (at your option) any later version.

;; This program is distributed in the hope that it will be useful,
;; but WITHOUT ANY WARRANTY; without even the implied warranty of
;; MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
;; GNU General Public License for more details.

;;; Commentary:
;; Emacs-клиент для Mattermost, использующий Rust-мост.
;; Сервер является единственным источником истины, буферы Emacs служат
;; только для отображения и ввода.

;;; Code:
(require 'mattermost-custom)
(require 'mattermost-files)
(require 'mattermost-protocol)
(require 'mattermost-dm)
(require 'mattermost-channel)
(require 'mattermost-dashboard)

(provide 'mattermost)
;;; mattermost.el ends here
