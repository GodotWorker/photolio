import { useEffect, useState } from 'react';
import { useParams, NavLink } from 'react-router-dom';
import { BlurHashWrapper } from 'components/BlurHashWrapper';
import { ImageModel } from 'models/ImageModel';
import ImageService from 'services/ImageService';
import SiteService, { GalleryModel } from 'services/SiteService';
import Masonry from 'react-masonry-css';
export const GalleryRoute: React.FC = () => {
  const { slug = '' } = useParams();
  const [images, setImages] = useState<ImageModel[]>([]);
  const [gallery, setGallery] = useState<GalleryModel>();
  const [error, setError] = useState('');
  useEffect(() => {
    let active = true;
    Promise.all([ImageService.list(slug), SiteService.getPublicSite()]).then(([photos, site]) => {
      if (active) { setImages(photos); setGallery(site.galleries.find((item) => item.slug === slug)); }
    }).catch((reason: Error) => { if (active) setError(reason.message); });
    return () => { active = false; };
  }, [slug]);
  if (error) return <main><p>{error}</p></main>;
  return <main style={{ padding: '0 20px' }}>
    <h1>{gallery?.title || 'Gallery'}</h1>{gallery?.description && <p>{gallery.description}</p>}
    {images.length === 0 ? <p>This gallery does not have any photos yet.</p> :
      <Masonry breakpointCols={{ default: 4, 1100: 3, 700: 2, 500: 1 }} className="gallery-masonry" columnClassName="gallery-masonry-column">
        {images.map((image) => <div key={image.id} style={{ marginBottom: 18 }}>
          <NavLink to={'/images/' + encodeURIComponent(image.id)}>
            <BlurHashWrapper image={image} width={260} imageURL={ImageService.getThumbnailSource(image.id, 260)} />
          </NavLink>
        </div>)}
      </Masonry>}
  </main>;
};