import {
  HttpContextToken,
  type HttpEvent,
  type HttpHandler,
  type HttpInterceptor,
  type HttpRequest,
} from '@angular/common/http';
import { inject, Service } from '@angular/core';
import { LoadingService } from './loading-indicator.service';
import { finalize, type Observable } from 'rxjs';

export const SkipLoading = new HttpContextToken<boolean>(() => false);

@Service()
export class LoadingInterceptor implements HttpInterceptor {
  loadingService = inject(LoadingService);

  intercept(req: HttpRequest<any>, next: HttpHandler): Observable<HttpEvent<any>> {
    // Can be used to avoid showing the loading spinner for certain requests
    if (req.context.get(SkipLoading)) {
      return next.handle(req);
    }

    this.loadingService.loadingOn();
    return next.handle(req).pipe(finalize(() => this.loadingService.loadingOff()));
  }
}
