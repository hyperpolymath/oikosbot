;;; SPDX-License-Identifier: MPL-2.0
;;; SPDX-FileCopyrightText: 2024-2025 hyperpolymath
;;;
;;; Oikos Bot Guix Manifest
;;;
;;; Development environment manifest for oikos-bot.
;;; Use with: guix shell -m manifest.scm

(specifications->manifest
 '(;; ========================================
   ;; Core Languages
   ;; ========================================

   ;; Haskell (for code analyzer)
   "ghc"
   "cabal-install"
   "hlint"
   "haskell-language-server"

   ;; OCaml (for documentation analyzer)
   "ocaml"
   "dune"
   "opam"
   "ocaml-merlin"
   "ocaml-ocp-indent"
   "ocamlformat"

   ;; ReScript (compiles from source, needs node for build)
   "node"  ; Only for rescript compiler, not runtime

   ;; Deno runtime
   "deno"

   ;; Python (for policy engine)
   "python"
   "python-pip"
   "python-virtualenv"

   ;; Rust (for orchestrator)
   "rust"
   "rust-analyzer"

   ;; ========================================
   ;; Datastore
   ;; ========================================

   ;; Single store is VeriSimDB (external; not yet packaged in Guix). The legacy
   ;; ArangoDB + Virtuoso dev tools were removed with the single-store migration;
   ;; runtime client wiring is deferred (see ROADMAP.adoc).

   ;; ========================================
   ;; Logic Programming
   ;; ========================================

   ;; Datalog (Souffle)
   "souffle"

   ;; Prolog (for DeepProbLog base)
   "swi-prolog"

   ;; ========================================
   ;; Build Tools
   ;; ========================================

   "git"
   "make"
   "gcc-toolchain"
   "pkg-config"
   "openssl"
   "zlib"

   ;; ========================================
   ;; Container Tools (Vörðr preferred)
   ;; ========================================

   "podman"
   "buildah"
   "skopeo"
   "cni-plugins"

   ;; ========================================
   ;; Development Utilities
   ;; ========================================

   "jq"
   "ripgrep"
   "fd"
   "bat"
   "direnv"
   "watchexec"))
