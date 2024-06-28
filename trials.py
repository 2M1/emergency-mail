#!/usr/bin/env python3
import smtplib
import ssl
import os
from email.utils import make_msgid
from email.message import EmailMessage
import random

PORT = 465
HOST = os.getenv("EM_IMAP_HOST")
USER = os.getenv("EM_IMAP_USERNAME")
PASS = os.getenv("EM_IMAP_PASSWORD")

ALARM_MESSAGES = [
    # "examples/emergency_bgebg_asciiescaped.txt",
    "examples/emergency_bgebg.txt",
    "examples/emergency_many_units.txt",
#    "examples/emergency_obj.txt",
#    "examples/emergency_r1n1f.txt",
#    "examples/emergency_simple.txt",
]


def establish_connection() -> smtplib.SMTP_SSL:
    print(f"Connecting to server as {USER} on {HOST}:{PORT}")
    context = ssl._create_unverified_context()
    server = smtplib.SMTP_SSL(host=HOST, port=PORT, context=context)
    server.ehlo()
    server.login(USER, PASS)
    print("Logged in")
    return server


def teardown_connection(con: smtplib.SMTP_SSL):
    print("Terminating connection")
    con.quit()


def send_alarmmail(connection: smtplib.SMTP_SSL, mail_path: str):
    with open(mail_path, "r", encoding="utf-8") as f:
        print("Sending mail")
        msg = EmailMessage()
        msg.set_content(f.read())
        msg["Subject"] = "Alarm"
        msg["From"] = USER
        msg["To"] = USER
        msg["Message-ID"] = make_msgid()
        print(msg.as_string())
        connection.send_message(msg)
    print("Mail sent")


def main():
    # sends two mails at the same time to test the application for multiple simultaneous alarms
    connection = establish_connection()
    for choice in random.choices(ALARM_MESSAGES, k=2):
        send_alarmmail(connection, choice)
    teardown_connection(connection)


if __name__ == "__main__":
    main()
