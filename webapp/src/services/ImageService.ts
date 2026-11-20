import { ImageModel } from 'models/ImageModel';
import { API_ROOT } from './SiteService';
export default class ImageService {
  public static list(gallery?: string): Promise<ImageModel[]> {
    const suffix = gallery ? '?gallery=' + encodeURIComponent(gallery) : '';
    return window.fetch(API_ROOT + '/images' + suffix).then((res) => {
      if (!res.ok) throw new Error('Could not load photos.');
      return res.json();
    });
  }
  public static getMeta(id: string): Promise<ImageModel> {
    return window.fetch(API_ROOT + '/images/' + encodeURIComponent(id) + '/meta').then((res) => res.json());
  }
  public static getImageSource(id: string): string {
    return API_ROOT + '/images/' + encodeURIComponent(id);
  }
  public static getThumbnailSource(id: string, width?: number): string {
    return API_ROOT + '/images/' + encodeURIComponent(id) + '/thumbnail?width=' + width;
  }
}