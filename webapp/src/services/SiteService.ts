export type GalleryModel = { slug: string; title: string; description: string; };
export type ActivityItem = { at: string; action: string; detail: string; };
export type SiteSettings = {
  title: string; tagline: string; about: string; contact_email: string; contact_phone: string;
  theme: string; website_url: string; instagram_url: string; youtube_url: string;
  facebook_url: string; tiktok_url: string; galleries: GalleryModel[]; activity?: ActivityItem[];
};
export const API_ROOT = process.env.REACT_APP_API_ROOT_URL ??
  (process.env.NODE_ENV === 'production' ? '/api' : 'http://localhost:5000');
export default class SiteService {
  public static getPublicSite(): Promise<SiteSettings> {
    return fetch(API_ROOT + '/site').then(async (response) => {
      if (!response.ok) throw new Error('Could not load website settings.');
      return response.json();
    });
  }
}