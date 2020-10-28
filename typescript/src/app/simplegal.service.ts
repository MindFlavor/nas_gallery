import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';
import { map } from 'rxjs/operators';
import { PreviewFile, Folder } from './entities/items';

@Injectable({
  providedIn: 'root'
})
export class SimplegalService {

  constructor(private http: HttpClient) {
  }

  public getPreview(prefix: string, path: string): Observable<PreviewFile[]> {
    const entries = this.http.get<PreviewFile[]>("http://10.100.1.80:8000/list/Preview" + path).pipe(map(items => items.sort((a, b) => a.path.localeCompare(b.path))));
    return entries;
  }

  public getFolders(path: string): Observable<Folder[]> {
    const entries = this.http.get<Folder[]>("http://10.100.1.80:8000/list/Folder/" + path).pipe(map(items => items.sort((a, b) => a.path.localeCompare(b.path))));
    return entries;
  }

  public getRootFolders(): Observable<string[]> {
    return this.http.get<string[]>("http://10.100.1.80:8000/firstlevel").pipe(map(items => items.sort()));
  }

  public isFolderAllowed(path: string): Observable<boolean> {
    return this.http.get<boolean>("http://10.100.1.80:8000/allowed/" + path);
  }
}
