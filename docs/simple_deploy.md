# Simple deploy

1. Clone the repo: `git clone https://github.com/MindFlavor/nas_gallery.git`
2. Build the image `cd nas_gallery && docker build -t nas_gallery .`
3. Copy the [docker-compose.yml simple file](https://github.com/MindFlavor/nas_gallery/blob/master/docker-compose/simple/docker-compose.yml) to a directory of your choice.
4. Edit the `docker-compose.yml` file: find the line mapping `- /your_folder:/mnt/nas` and replace `/your_folder` with your path (you might need to change port if it's already in use).
5. Start the container with `docker-compose up`. 
6. Browse http::<your_server_ip> and profit!

> Step 1 and 2 will become unnecessary after I have published the image in the public docker registry.

This setup is fine for an internal networks **only** since there is no authentication. 
