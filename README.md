# NAS picture gallery

# Why?

So you have a lot of pictures in your NAS and no way to share them? Did you take a lot of pictures at last party and want to share them to other people *without* resorting to a social network? Would you like to browse your pictures from a handheld device *without* uploading them to the cloud? If yes, and you have some basic docker skills, this program is for you. 

NAS Gallery does just what you expect it to do: you point it to a folder and it creates a browsable gallery of its pictures and videos. Simple and effective. It also allows you to browse the subdirectories (you spent days organizing your photos, didn't you?). 

NAS Gallery also allows you, optionally, you to specify who can access each folder: this way you can safely share the kid's pictures with your in-laws while keeping the "other" pictures for yourself only. The configuration can be as complex as you like.

# How

NAS Gallery is comprised of three components: 

1. A REST back-end written in Rust ([Rocket](https://rocket.rs/)). The back-end handles the creation of thumbnails and the enforcement of the ACLs. It also streams the files as needed so you can serve large video files without having an enterprise-class computer. Also, being Rust, it's pretty fast.
2. An Angular front-end for the data visualization. At this stage the front-end does not use Angular routing but care has been taken to allow permanent links. This way you can share a URL of a picture and the recipient will open it (provided they have the proper authorization of course).
3. An authentication proxy. The proxy is [oauth2-proxy](https://github.com/oauth2-proxy/oauth2-proxy) - an external unaffiliated project. The proxy handles authentication so NAS Gallery does not have to do it itself. It also means we are using a great, secure program for this sensitive task!

The authentication proxy is optional. If you do not care about authentication and authorization you can freely skip it as demonstrated in [simple deploy](docs/simple_deploy.md).

![logical diagram](docs/diag.png)
