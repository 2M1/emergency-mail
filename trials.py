import smtplib
import ssl
import os

PORT = 993
HOST = "aquila.uberspace.de"
USER = os.getenv("EM_IMAP_USERNAME")
PASS = os.getenv("EM_IMAP_PASSWORD")

MAIL_PATH = "examples/emergency_bgebg.txt"

def main():
    context = ssl.create_default_context()

    with smtplib.SMTP(HOST, PORT) as server:
        server.ehlo()
        server.starttls(context=context)
        server.ehlo()
        server.login(USER, PASS)

        with open(MAIL_PATH, "r") as f:
            server.sendmail(USER, USER, f.read())

if __name__ == "__main__":
    main()