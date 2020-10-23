import { TestBed } from '@angular/core/testing';

import { SimplegalService } from './simplegal.service';

describe('SimplegalService', () => {
  let service: SimplegalService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(SimplegalService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
