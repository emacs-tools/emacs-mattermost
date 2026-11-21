.PHONY: lint test build clean

EMACS ?= emacs

lint:
	$(EMACS) -Q --batch -L lisp \
	--eval "(progn \
		(require 'package) \
		(package-initialize) \
		(unless (package-installed-p 'package-lint) \
			(package-refresh-contents) \
			(package-install 'package-lint)) \
		(setq package-lint-main-file \"lisp/mattermost.el\") \
		(package-lint-batch-and-exit))"

test:
	$(EMACS) -Q --batch -L lisp -L tests \
	--eval "(require 'ert)" \
	--eval "(ert-run-tests-batch-and-exit)"

build:
	docker compose build

clean:
	docker compose down --volumes --remove-orphans
	rm -rf data/
