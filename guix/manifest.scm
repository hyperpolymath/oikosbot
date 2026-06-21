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

   ;; Haskell (analyzers/code-haskell)
   "ghc"
   "cabal-install"
   "hlint"
   "haskell-language-server"

   ;; Rust (crates/oikosbot-* analysis workspace + CLI)
   "rust"
   "rust-analyzer"

   ;; Deno — default backend + runtime for the AffineScript bot.
   ;; NOTE: the AffineScript compiler is external (built in the
   ;; hyperpolymath/affinescript repo); point AS_BIN at it. ReScript was
   ;; retired (oikos#41) and Python is not used here — the policy engine is
   ;; interpreted Datalog/DeepProbLog (souffle + swi-prolog, below).
   "deno"

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
