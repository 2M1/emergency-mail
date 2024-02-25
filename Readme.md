# Programm zum automatischen Abrufen und Drucken von Einsatz Emails

![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/2M1/emergency-mail/Rust?label=tests)

Work in progress.

## TODO
- [x] Parse unit table based on table headers
- [x] generate pdf (mostly working)
- [x] print with sumatraPDF or Adobe (both cli) (sumatra working)
- [ ] notify user/admin on error (add text to printout or popup warning or email)
- [x] print times the number of alarmed engines from the amt, also considers county and organisation (FL)
- [x] make windows support a compile feature to also be able to build on linux (linux as a seperate feature switch?)

## possible futures:
- [ ] add gui for status and reprint of last n Ems
- [ ] add config generation to gui

## on server
- ubuntu 22.04
  - usb stick (silver) ist registriert, löschen und evtl. formatieren, so dass er wieder geht :)

## other:
use .env file:
```bash
export $(cat .env | xargs)
```