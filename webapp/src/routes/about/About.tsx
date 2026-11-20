import { useEffect, useState } from 'react';
import { ALink } from 'components/ALink';
import { PaddingContainer } from 'components/PaddingContainer';
import SiteService, { SiteSettings } from 'services/SiteService';
export const AboutRoute: React.FC = () => {
  const [site, setSite] = useState<SiteSettings>();
  useEffect(() => { SiteService.getPublicSite().then(setSite).catch(() => undefined); }, []);
  const links: [string, string][] = [
    ['Website', site?.website_url || ''], ['Instagram', site?.instagram_url || ''],
    ['YouTube', site?.youtube_url || ''], ['Facebook', site?.facebook_url || ''], ['TikTok', site?.tiktok_url || ''],
  ].filter((item): item is [string, string] => Boolean(item[1]));
  return <PaddingContainer><div style={{ textAlign: 'justify', padding: '0 30px' }}>
    <h1>About {site?.title || 'me'}</h1><p>{site?.about || 'More information coming soon.'}</p>
    {links.length > 0 && <><h2>Find me online</h2><div style={{ display: 'flex', flexWrap: 'wrap', gap: 18 }}>
      {links.map(([label, url]) => <ALink key={label} href={url}>{label}</ALink>)}
    </div></>}
  </div></PaddingContainer>;
};