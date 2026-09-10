$id-6903637887272960
title: First-class CLI interface
date: 2026/09/10
source: @ottojung
kind: requirement

The CLI is a first-class interface. There must be a simple deploy command that takes exactly the Git repository, exact Git commit, repository subdirectory, and namespace name (plus only truly necessary optional flags such as builder override if config requires it). Deploying should ensure/create/configure that namespace administratively as needed, build the specified source via make build using the configured shared builder container, create an immutable release, and atomically activate it for /a/<namespace>/.
