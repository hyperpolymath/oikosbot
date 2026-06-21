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
   "haskell-language-server"   ; not in guix 1.4.0 — verify in current guix

   ;; Rust (crates/oikosbot-* analysis workspace + CLI)
   "rust"
   "rust-analyzer"

   ;; Deno — the AffineScript bot's default backend + runtime. NOTE: Deno is
   ;; NOT packaged in Guix, so it cannot be a manifest spec; install it
   ;; separately (https://deno.land) or via a custom channel. The AffineScript
   ;; compiler is likewise external (built in hyperpolymath/affinescript; point
   ;; AS_BIN at it). ReScript was retired (oikos#41) and Python is not used —
   ;; the policy engine is interpreted Datalog/DeepProbLog (souffle + swi-prolog).
   ;; "deno"   ; unpackaged in Guix — left commented so `guix shell -m` works

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
   "souffle"   ; not in guix 1.4.0 — verify in current guix

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
   "buildah"   ; not in guix 1.4.0 — verify in current guix
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
