;;; SPDX-License-Identifier: MPL-2.0
;;; manifest.scm — Generic Guix manifest for RSR-compliant projects
;;;
;;; Usage:
;;;   guix shell -m manifest.scm
;;;

(specifications->manifest
  '(;; Core development tools
    "git"
    "just"      ; not in guix 1.4.0 — verify in current guix
    "nickel"    ; not in guix 1.4.0 — verify in current guix
    "curl"
    "bash"
    "coreutils"

    ;; Documentation
    "ruby-asciidoctor"   ; guix packages the asciidoctor gem as `ruby-asciidoctor`
    "pandoc"

    ;; Common build dependencies
    "openssl"
    "zlib"
    "pkg-config"))
