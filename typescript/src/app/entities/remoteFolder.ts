import { encode } from 'punycode';

export class RemoteFolder {
    constructor(public path: string) { }

    public encodedPath(): string {
        const encodedPar = this.path.replace('(', '%op').replace(')', "%cp");
        const encoded = encodeURI(encodedPar);
        //if (encoded != this.path)
        //    console.log(`encoded == ${encoded}`);

        return encoded;
    }

    static decodePath(path: string): string {
        const decodedURI = decodeURI(path);
        const decoded = decodedURI.replace('%op', '(').replace("%cp", ')');
        //if (decoded != decodedURI)
        //    console.log(`dencoded == ${decoded}`);

        return decoded;
    }

    public lastPathItem(): string {
        const lastPathSeparator = this.path.lastIndexOf('/');
        if (lastPathSeparator) {
            return this.path.substring(lastPathSeparator + 1);
        } else {
            return this.path;
        }
    }

}