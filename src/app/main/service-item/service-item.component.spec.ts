import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ServiceItemComponent } from './service-item.component';

describe('ServiceItemComponent', () => {
  let component: ServiceItemComponent;
  let fixture: ComponentFixture<ServiceItemComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ServiceItemComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(ServiceItemComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
