import { ComponentFixture, TestBed } from '@angular/core/testing';

import { WhisperComponent } from './whisper.component';

describe('WhisperComponent', () => {
  let component: WhisperComponent;
  let fixture: ComponentFixture<WhisperComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [WhisperComponent]
    })
    .compileComponents();
    
    fixture = TestBed.createComponent(WhisperComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
