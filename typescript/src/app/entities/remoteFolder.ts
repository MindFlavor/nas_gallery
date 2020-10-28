import { encode } from 'punycode';

export class RemoteFolder {
    constructor(public path: string) { }

    public encodedPath(): string {
        console.log(`path to encode  == ${this.path}`);
        const encodedPar = this.path.replace('(', '%op').replace(')', "%cp");
        console.log(`encodedPar == ${encodedPar}`);
        const encoded = encodeURI(encodedPar);
        console.log(`encoded final == ${encoded}`);
        //if (encoded != this.path)
        //    console.log(`encoded == ${encoded}`);

        return encoded;
    }

    static decodePath(path: string): string {
        console.log(`received path == ${path}`);
        const decodedURI = decodeURI(path);
        console.log(`decoded URI == ${decodedURI}`);
        const decodedURI2 = decodeURI(decodedURI);
        console.log(`decoded URI == ${decodedURI2}`);
        const decoded = decodedURI2.replace('%op', '(').replace("%cp", ')');
        console.log(`decoded final == ${decoded}`);
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