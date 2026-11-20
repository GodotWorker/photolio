import { Link, useNavigate } from 'react-router-dom';
import { useEffect, useState } from 'react';
import SiteService, { GalleryModel } from 'services/SiteService';
export const Header: React.FC = () => {
  const navigate = useNavigate();
  const [title, setTitle] = useState('Photolio');
  const [galleries, setGalleries] = useState<GalleryModel[]>([]);
  useEffect(() => {
    SiteService.getPublicSite().then((site) => { setTitle(site.title); setGalleries(site.galleries); }).catch(() => undefined);
  }, []);
  return <header style={{ padding: '12px 12px 28px', display: 'flex', justifyContent: 'center', alignItems: 'center', flexWrap: 'wrap', gap: 24 }}>
    <div onClick={() => navigate('/')} style={{ cursor: 'pointer', textAlign: 'center' }}>
      <h1 style={{ margin: 0, fontSize: 38 }}>{title}</h1><div style={{ letterSpacing: '.2em', fontSize: 11, textTransform: 'uppercase' }}>Photography</div>
    </div>
    <nav style={{ display: 'flex', gap: 16, flexWrap: 'wrap', justifyContent: 'center', alignItems: 'center' }}>
      <Link to="/">Home</Link><Link to="/about">About</Link><Link to="/contact">Contact</Link>
      {galleries.map((gallery) => <Link key={gallery.slug} to={'/gallery/' + encodeURIComponent(gallery.slug)}>{gallery.title}</Link>)}
      <Link to="/admin">Admin</Link>
    </nav>
  </header>;
};