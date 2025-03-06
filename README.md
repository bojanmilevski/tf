# TF

Recursively transfer media files.

## HOW

you provide your source folder, where all of your media files are located. `tf` moves these media
files to the provided destination folder, creating an appropriate media folder, an optional person
folder and year and month folders - based on the mtime of the file.

## EXAMPLE

source: /home/user/files
dst: /home/my_user/media
person: me

```
/home/user/files/
|_ picture.png                  (mtime: 01.01.2000)
|_ video.mp4                    (mtime: 30.10.2004)
|_ notes.txt
|_ folder/
   |_ another_video.mkv         (mtime: 10.12.2009)
   |_ another_picture.jpg       (mtime: 04.05.2024)
   |_ another_note.odt
```

/home/user/files/picture.png -> /home/my_user/media/pictures/me/2000/january/picture.png
/home/user/files/video.mp4 -> /home/my_user/media/videos/me/2004/october/video.mp4
/home/user/files/folder/another_video.mkv -> /home/my_user/media/video/me/2009/december/another_video.mkv
/home/user/files/folder/another_picture.jpg -> /home/my_user/media/pictures/me/2024/may/another_picture.jpg
