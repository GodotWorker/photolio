import { useEffect, useState } from 'react';
import { BlurHashWrapper } from 'components/BlurHashWrapper';
import { ImageModel } from 'models/ImageModel';
import ImageService from 'services/ImageService';
import SiteService, { GalleryModel } from 'services/SiteService';
import Masonry from 'react-masonry-css';
import { NavLink } from 'react-router-dom';

type ImageGroups = [string, ImageModel[]][];
const groupImages = (images: ImageModel[]) => {
  const map: { [key: string]: ImageModel[] } = {};
  const keys: string[] = [];
  images.forEach((img) => {
    const group = img.group ?? '';
    if (map[group]) map[group].push(img);
    else { keys.push(group); map[group] = [img]; }
  });
  const groups: ImageGroups = keys.map((key) => [key, map[key]]);
  const empty = groups.findIndex(([group]) => group === '');
  if (empty >= 0) groups.push(...groups.splice(empty, 1));
  return groups;
};
export const HomeRoute: React.FC = () => {
  const [images, setImages] = useState<ImageGroups>();
  const [galleries, setGalleries] = useState<GalleryModel[]>([]);
  const [title, setTitle] = useState('Gallery');
  useEffect(() => {
    Promise.all([ImageService.list(), SiteService.getPublicSite()]).then(([photos, site]) => {
      setImages(groupImages(photos)); setGalleries(site.galleries); setTitle(site.title);
    }).catch(() => setImages([]));
  }, []);
  const sections = images?.map(([slug, photos]) => {
    const gallery = galleries.find((item) => item.slug === slug);
    return <section key={slug || 'root'} style={{ marginBottom: 28, maxWidth: '100%' }}>
      <h2 style={{ opacity: .7, fontWeight: 300, fontSize: 20 }}>{gallery?.title ?? (slug || 'Other photos')}</h2>
      {gallery?.description && <p>{gallery.description}</p>}
      <Masonry breakpointCols={{ default: 4, 1100: 3, 700: 2, 500: 1 }} className="gallery-masonry" columnClassName="gallery-masonry-column">
        {photos.map((photo) => <div key={photo.id} style={{ marginBottom: 18 }}>
          <NavLink to={'/images/' + encodeURIComponent(photo.id)}>
            <BlurHashWrapper image={photo} width={250} imageURL={ImageService.getThumbnailSource(photo.id, 250)} />
          </NavLink>
        </div>)}
      </Masonry>
      {slug && <NavLink to={'/gallery/' + encodeURIComponent(slug)}>View this gallery</NavLink>}
    </section>;
  });
  return <main style={{ padding: '0 18px', maxWidth: '100%', width: 'fit-content', margin: '0 auto' }}>
    <h1>{title}</h1>
    {galleries.map((gallery) => <p key={gallery.slug}><NavLink to={'/gallery/' + encodeURIComponent(gallery.slug)}>{gallery.title} →</NavLink></p>)}
    {sections || <p>Loading photos…</p>}
  </main>;
};