;;; SPDX-License-Identifier: MPL-2.0
;;; SPDX-FileCopyrightText: 2024-2025 hyperpolymath
;;;
;;; Oikos Bot Guix Package Definition
;;;
;;; This defines the oikos-bot package for Guix.

(define-module (oikos)
  #:use-module (guix packages)
  #:use-module (guix git-download)
  #:use-module (guix build-system gnu)
  #:use-module (guix build-system haskell)
  #:use-module ((guix licenses) #:prefix license:)
  #:use-module (gnu packages haskell)
  #:use-module (gnu packages haskell-xyz))

;; Haskell Code Analyzer
(define-public oikos-analyzer-haskell
  (package
    (name "oikos-analyzer-haskell")
    (version "0.1.0")
    (source
     (origin
       (method git-fetch)
       (uri (git-reference
             (url "https://github.com/hyperpolymath/oikosbot")
             (commit (string-append "v" version))))
       (file-name (git-file-name name version))
       (sha256
        (base32 "0000000000000000000000000000000000000000000000000000"))))
    (build-system haskell-build-system)
    (inputs
     (list ghc-aeson
           ghc-text
           ghc-containers
           ghc-vector
           ghc-mtl
           ghc-optparse-applicative
           ghc-megaparsec))
    (arguments
     '(#:cabal-file "analyzers/code-haskell/oikos-analyzer.cabal"))
    (synopsis "Haskell code analyzer for Oikos Bot")
    (description
     "Analyzes code for carbon intensity, energy efficiency,
      Pareto optimality, and software quality metrics.")
    (home-page "https://github.com/hyperpolymath/oikosbot")
    (license license:mpl2.0)))

;; NOTE: The OCaml "docs" analyzer package was removed — analyzers/docs-ocaml
;; does not exist in this repo. The policy engine is interpreted Datalog +
;; DeepProbLog rules (policy-engine/datalog, policy-engine/deepproblog), not a
;; built package; its interpreters (souffle, swi-prolog) live in manifest.scm.

;; Combined oikos-bot package
(define-public oikos-bot
  (package
    (name "oikos-bot")
    (version "0.1.0")
    (source #f)
    (build-system gnu-build-system)
    (inputs
     ;; Datastore is VeriSimDB (external; not yet packaged in Guix). The legacy
     ;; ArangoDB + Virtuoso inputs were removed with the single-store migration;
     ;; the runtime client is deferred (see ROADMAP.adoc). Deno is a dev/runtime
     ;; tool (guix/manifest.scm), not a package build input.
     (list oikos-analyzer-haskell))
    (arguments
     '(#:phases
       (modify-phases %standard-phases
         (delete 'configure)
         (delete 'build)
         (replace 'install
           (lambda* (#:key outputs #:allow-other-keys)
             (let ((out (assoc-ref outputs "out")))
               ;; Create wrapper scripts
               (mkdir-p (string-append out "/bin"))
               #t))))))
    (synopsis "Ecological & Economic Code Analysis Platform")
    (description
     "Oikos Bot analyzes code for ecological soundness and economic
      efficiency using Pareto optimality and allocative efficiency
      as normative criteria.")
    (home-page "https://github.com/hyperpolymath/oikosbot")
    (license license:mpl2.0)))
