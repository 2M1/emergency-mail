# Programm zum automatischen Abrufen und Drucken von Einsatz Emails

![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/2M1/emergency-mail/Rust?label=tests)

Work in progress.

## TODO
- [x] Parse unit table based on table headers
- [-] generate pdf (mostly working)
- [-] print with sumatraPDF or Adobe (both cli) (sumatra working)
- [ ] notify user/admin on error (add text to printout or popup warning or email)
- [x] print times the number of alarmed engines from the amt, also considers county and organisation (FL)

## possible futures:
- [ ] add gui for status and reprint of last n Ems
- [ ] add config generation to gui