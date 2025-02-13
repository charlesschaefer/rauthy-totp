import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ServiceEditComponent } from './service-edit.component';

describe('ServiceEditComponent', () => {
  let component: ServiceEditComponent;
  let fixture: ComponentFixture<ServiceEditComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ServiceEditComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(ServiceEditComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
