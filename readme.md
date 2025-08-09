# Bulk Upload GDrive

Gm, this is a script that I wrote to upload stuff to google drive in bulk. Eg. you have a thousands of files and want to speed up the process you could use this. I was able to upload 1881 pdfs, around 12 gb in total, in 16 minutes. On the gdrive web app it would have taken hours. Not just 1 or 2, but many.

If you're wondering about the costs then this script uses the `google.apps.drive.v3.DriveFiles.Create` API. This doesn't cost anything. Its rate limits are 12k requests per minute, plenty imo.
