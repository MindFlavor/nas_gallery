import { Component, OnInit } from '@angular/core';
import { ActivatedRoute, Router, Event, NavigationStart, NavigationEnd, NavigationError } from '@angular/router';
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
  public path: string;

  constructor(private simplegal: SimplegalService, private route: ActivatedRoute, private router: Router) { }

  ngOnInit(): void {
    this.loadFolder(RemoteFolder.decodePath(window.location.pathname + window.location.search));

    this.router.events.subscribe((event: Event) => {
      if (event instanceof NavigationStart) {
        this.loadFolder(RemoteFolder.decodePath(event.url));
      }
    });
  }

  private loadFolder(path: string) {
    this.isParentAllowed = false;
    this.isRoot = true;
    this.path = path;
    this.currentPage = 0;
    this.subFolders = [];

    // handle query param, if present
    {
      if (this.path.indexOf('?') != -1) {
        this.currentPage = +path.substr(path.indexOf('=') + 1);
        this.path = path.substr(0, path.indexOf('?'));
      }
    }

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

  public encodePath(path: string): String {
    return new RemoteFolder(path).encodedPath();
  }

  public getUpperFolder(): String {
    //const path = RemoteFolder.decodePath(window.location.pathname);
    const idx = this.path.lastIndexOf("/");
    return this.path.substring(0, idx);
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

  public lastFolder(path: string): string {
    const lastPathSeparator = path.lastIndexOf('/');
    if (lastPathSeparator) {
      return path.substring(lastPathSeparator + 1);
    } else {
      return path;
    }
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
