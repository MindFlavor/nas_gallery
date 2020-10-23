import { Component, OnInit } from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { encode } from 'punycode';
import { max } from 'rxjs/operators';
import { PreviewFile } from '../entities/items';
import { PageLink } from '../entities/pagelink';
import { RemoteFolder } from '../entities/remoteFolder';
import { SimplegalService } from '../simplegal.service';

@Component({
  selector: 'app-preview',
  templateUrl: './preview.component.html',
  styleUrls: ['./preview.component.css']
})
export class PreviewComponent implements OnInit {
  readonly ITEMS_PER_PAGE = 20;
  readonly MAX_PAGES = 7;
  public previewFiles: PreviewFile[];
  public rootFolders: String[] = null;
  public subFolders: RemoteFolder[] = null;
  public currentPage: number;
  public isParentAllowed: boolean;
  public isRoot: boolean;

  constructor(private simplegal: SimplegalService, private route: ActivatedRoute) { }

  ngOnInit(): void {
    this.isParentAllowed = false;
    this.isRoot = true;

    const path = RemoteFolder.decodePath(window.location.pathname);
    this.currentPage = 0;
    if (window.location.search.indexOf('=') != -1)
      this.currentPage = +window.location.search.substr(window.location.search.indexOf('=') + 1);
    this.subFolders = [];

    this.simplegal.getRootFolders().subscribe(rootFolders => {
      console.log(`got root folders ${rootFolders}`);
      this.rootFolders = rootFolders;
      if (path == '/') {
        this.subFolders = rootFolders.map(folder => new RemoteFolder(folder));
      }
    });

    // if we are in the root, do not request previews
    if (path != '/') {
      this.isRoot = false;
      console.log(`requesting path ${path}`);

      this.simplegal.getPreview(window.location.host, path).subscribe(entries => {
        this.previewFiles = entries;
      });

      this.simplegal.getFolders(path).subscribe(folders => {
        this.subFolders = folders.map(folder => new RemoteFolder(folder.path));
      });

      const lastPathSeparator = path.lastIndexOf("/");
      this.simplegal.isFolderAllowed(path.substring(0, lastPathSeparator)).subscribe(v => {
        console.log('parent allowed == ' + v);
        this.isParentAllowed = v;
      });
    }
  }

  public filesForThisPage(): PreviewFile[] {
    if (!this.previewFiles)
      return [];
    const items = this.previewFiles.slice(this.currentPage * this.ITEMS_PER_PAGE, this.currentPage * this.ITEMS_PER_PAGE + this.ITEMS_PER_PAGE);
    return items;
  }

  public getPageLink(page: number): String {
    return window.location.pathname + `?p=${page}`;
  }

  public getUpperFolder(): String {
    const path = RemoteFolder.decodePath(window.location.pathname);
    const idx = path.lastIndexOf("/");
    return path.substring(0, idx);
  }

  public pages(): PageLink[] {
    if (!this.previewFiles)
      return [];
    let pageLinks = [];
    const totalPages = Math.ceil(this.previewFiles.length / this.ITEMS_PER_PAGE);
    if (totalPages < this.MAX_PAGES) {
      for (let i = 0; i < totalPages; i++) {
        const toHyperlink = i != this.currentPage;
        pageLinks.push(new PageLink(i, i.toString(), toHyperlink));
      }
    } else {
      const itemsToShow = this.MAX_PAGES - 3;
      const maxDelta = Math.floor(itemsToShow / 2);

      if (this.currentPage > maxDelta) {
        // output 0 then ... and then the previous page and then the current page
        pageLinks.push(new PageLink(0, "0", true));
        pageLinks.push(new PageLink(this.currentPage - maxDelta * 2, "...", true));
        // publish maxdelta
        for (let i = this.currentPage - maxDelta; i < this.currentPage; i++) {
          pageLinks.push(new PageLink(i, i.toString(), true));
        }
      } else {
        // output the pages from beginning to current page
        for (let i = 0; i < this.currentPage; i++) {
          pageLinks.push(new PageLink(i, i.toString(), true));
        }
      }

      //output current page
      pageLinks.push(new PageLink(this.currentPage, this.currentPage.toString(), false));

      if (totalPages - this.currentPage > maxDelta) {
        // publish maxdelta
        for (let i = this.currentPage + 1; i < this.currentPage + maxDelta; i++) {
          pageLinks.push(new PageLink(i, i.toString(), true));
        }
        pageLinks.push(new PageLink(this.currentPage + maxDelta * 2, "...", true));
        pageLinks.push(new PageLink(totalPages, totalPages.toString(), true));
      } else {
        // output the pages from next to total
        for (let i = this.currentPage + 1; i < totalPages + 1; i++) {
          pageLinks.push(new PageLink(i, i.toString(), true));
        }
      }
    }

    return pageLinks;
  }

  public asRows(): PreviewFile[][] {
    if (!this.previewFiles) return null;

    let rows: PreviewFile[][] = [];
    let cnt = 0;
    let currentRow: PreviewFile[] = null;

    for (let item of this.previewFiles) {
      if (!currentRow) {
        currentRow = [];
        cnt = 0;
      }

      currentRow.push(item);
      cnt++;

      if (cnt == 5) {
        rows.push(currentRow);
        currentRow = null;
      }
    }

    if (!currentRow) rows.push(currentRow);

    return rows;
  }

  public lastFolder(path: string): string {
    const lastPathSeparator = path.lastIndexOf('/');
    if (lastPathSeparator) {
      return path.substring(lastPathSeparator + 1);
    } else {
      return path;
    }
  }

  public currentPath(): string {
    return RemoteFolder.decodePath(window.location.pathname);
  }

  public lastPathItem(path: string): string {
    const lastPathSeparator = path.lastIndexOf('/');
    if (lastPathSeparator) {
      return path.substring(lastPathSeparator + 1);
    } else {
      return path;
    }
  }
}
