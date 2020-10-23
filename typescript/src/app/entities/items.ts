export class Folder {
    constructor(public path: string) { }

}

export class File extends Folder {
    constructor(
        path: string,
        public size: number
    ) { super(path); }
}

export class PreviewFile extends File { }