import { useEffect, useState } from 'react';
import { ALink } from 'components/ALink';
import { PaddingContainer } from 'components/PaddingContainer';
import SiteService, { SiteSettings } from 'services/SiteService';
export const ContactRoute: React.FC = () => {
  const [site, setSite] = useState<SiteSettings>();
  useEffect(() => { SiteService.getPublicSite().then(setSite).catch(() => undefined); }, []);
  const links: [string, string][] = [
    ['Website', site?.website_url || ''], ['Instagram', site?.instagram_url || ''],
    ['YouTube', site?.youtube_url || ''], ['Facebook', site?.facebook_url || ''], ['TikTok', site?.tiktok_url || ''],
  ].filter((item): item is [string, string] => Boolean(item[1]));
  return <PaddingContainer><div style={{ padding: '0 20px' }}>
    <h1>Get in touch</h1><p>For enquiries, print requests or bookings, use the contact details below.</p>
    {site?.contact_email && <p>Email: <a href={'mailto:' + site.contact_email}>{site.contact_email}</a></p>}
    {site?.contact_phone && <p>Phone: <a href={'tel:' + site.contact_phone}>{site.contact_phone}</a></p>}
    {links.length > 0 && <><h2>Social media and links</h2><div style={{ display: 'flex', flexWrap: 'wrap', gap: 18 }}>
      {links.map(([label, url]) => <ALink key={label} href={url}>{label}</ALink>)}
    </div></>}
  </div></PaddingContainer>;
};