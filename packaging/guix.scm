;; SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
;; SPDX-License-Identifier: GPL-3.0-or-later
;;
;; GNU Guix package definition for wiremix (Standard §5.5).
;;
;; Rust applications in Guix require every crate dependency to be expressed as
;; a Guix package under #:cargo-inputs. At release time, regenerate the full
;; input set with:
;;
;;     guix import crate -r wiremix@0.11.0
;;
;; and splice the resulting rust-* package definitions into #:cargo-inputs
;; below. Then verify with:  guix build -f packaging/guix.scm

(use-modules (guix packages)
             (guix git-download)
             (guix build-system cargo)
             ((guix licenses) #:prefix license:)
             (gnu packages crates-io)
             (gnu packages linux)        ;pipewire
             (gnu packages llvm)         ;clang (bindgen)
             (gnu packages pkg-config)
             (gnu packages texinfo))

(package
  (name "wiremix")
  (version "0.11.0")
  (source
   (origin
     (method git-fetch)
     (uri (git-reference
           (url "https://github.com/Spacecraft-Software/wiremix")
           (commit (string-append "v" version))))
     (file-name (git-file-name name version))
     ;; Replace at release time:  guix hash -rx .
     (sha256
      (base32 "0000000000000000000000000000000000000000000000000000"))))
  (build-system cargo-build-system)
  (arguments
   (list
    #:install-source? #f
    ;; TODO(release): populate via `guix import crate -r wiremix`.
    #:cargo-inputs '()
    #:cargo-development-inputs '()
    #:phases
    #~(modify-phases %standard-phases
        (add-after 'build 'build-manual
          (lambda _
            (invoke "make" "info")))
        (add-after 'install 'install-manual
          (lambda* (#:key outputs #:allow-other-keys)
            (let* ((out (assoc-ref outputs "out"))
                   (info (string-append out "/share/info"))
                   (apps (string-append out "/share/applications")))
              (install-file "doc/wiremix.info" info)
              (install-file "wiremix.desktop" apps)))))))
  (native-inputs (list clang pkg-config texinfo))
  (inputs (list pipewire))
  (home-page "https://Wiremix.SpacecraftSoftware.org/")
  (synopsis "Dual-mode (TUI + agent-native CLI) mixer for PipeWire")
  (description
   "wiremix is a dual-mode mixer for PipeWire: an interactive terminal user
interface for humans, and a machine-readable, agent-native command-line
interface for scripts and automation.  It adjusts volumes, routes audio between
devices and applications, and configures device profiles and ports.")
  (license license:gpl3+))
