import { FormEvent, useEffect, useState } from 'react';
import { API_ROOT, ActivityItem, GalleryModel, SiteSettings } from 'services/SiteService';

type Dashboard = { photo_count: number; gallery_count: number; used_bytes: number; limit_bytes: number; activity: ActivityItem[]; };
const box: React.CSSProperties = { border: '1px solid rgba(127,127,127,.3)', borderRadius: 12, padding: 20, margin: '18px 0', background: 'rgba(127,127,127,.07)' };
const field: React.CSSProperties = { display: 'flex', flexDirection: 'column', gap: 6, margin: '10px 0', minWidth: 180, flex: 1 };
const control: React.CSSProperties = { width: '100%', padding: 10, border: '1px solid #9998', borderRadius: 7, background: 'transparent', color: 'inherit', font: 'inherit' };
const button: React.CSSProperties = { padding: '10px 16px', margin: '8px 8px 0 0', border: 0, borderRadius: 7, background: '#3d7569', color: 'white', fontWeight: 600, cursor: 'pointer' };
const row: React.CSSProperties = { display: 'flex', gap: 14, flexWrap: 'wrap' };
const muted: React.CSSProperties = { opacity: .7, fontSize: 13 };
const emptySettings: SiteSettings = { title: '', tagline: '', about: '', contact_email: '', contact_phone: '', theme: 'classic', website_url: '', instagram_url: '', youtube_url: '', facebook_url: '', tiktok_url: '', galleries: [] };
const prettyBytes = (bytes: number) => {
  const units = ['B', 'KB', 'MB', 'GB', 'TB']; let value = bytes || 0; let unit = 0;
  while (value >= 1000 && unit < units.length - 1) { value /= 1000; unit++; }
  return value.toFixed(unit ? 1 : 0) + ' ' + units[unit];
};

export const AdminRoute: React.FC = () => {
  const [token, setToken] = useState(() => window.localStorage.getItem('photolioAdminToken') || '');
  const [loginKey, setLoginKey] = useState('');
  const [settings, setSettings] = useState<SiteSettings>(emptySettings);
  const [dashboard, setDashboard] = useState<Dashboard>();
  const [galleryTitle, setGalleryTitle] = useState('');
  const [gallerySlug, setGallerySlug] = useState('');
  const [galleryDescription, setGalleryDescription] = useState('');
  const [files, setFiles] = useState<FileList | null>(null);
  const [uploadGallery, setUploadGallery] = useState('');
  const [error, setError] = useState('');
  const [notice, setNotice] = useState('');
  const [busy, setBusy] = useState(false);

  const api = async (path: string, init: RequestInit = {}, key = token) => {
    const response = await fetch(API_ROOT + path, {
      ...init,
      headers: { ...(init.headers || {}), Authorization: 'Bearer ' + key },
    });
    if (!response.ok) throw new Error((await response.text()) || ('Request failed: ' + response.status));
    if (response.status === 204) return null;
    return response.json();
  };
  const refresh = async (key: string) => {
    const results = await Promise.all([api('/admin/site', {}, key), api('/admin/dashboard', {}, key)]);
    setSettings(results[0] as SiteSettings);
    setDashboard(results[1] as Dashboard);
    setUploadGallery((current) => current || (results[0] as SiteSettings).galleries[0]?.slug || '');
  };

  useEffect(() => {
    if (!token) return;
    let active = true;
    Promise.all([
      fetch(API_ROOT + '/admin/site', { headers: { Authorization: 'Bearer ' + token } }),
      fetch(API_ROOT + '/admin/dashboard', { headers: { Authorization: 'Bearer ' + token } }),
    ]).then(async (responses) => {
      if (!responses[0].ok || !responses[1].ok) throw new Error('The admin key was rejected.');
      const values = await Promise.all([responses[0].json(), responses[1].json()]);
      if (active) {
        setSettings(values[0]); setDashboard(values[1]);
        setUploadGallery(values[0].galleries[0]?.slug || '');
        window.localStorage.setItem('photolioAdminToken', token);
      }
    }).catch((reason: Error) => {
      if (active) { setError(reason.message); setToken(''); window.localStorage.removeItem('photolioAdminToken'); }
    });
    return () => { active = false; };
  }, [token]);

  const login = async (event: FormEvent) => {
    event.preventDefault(); setBusy(true); setError('');
    try {
      await refresh(loginKey);
      window.localStorage.setItem('photolioAdminToken', loginKey);
      setToken(loginKey); setNotice('Signed in successfully.');
    } catch (reason) { setError(reason instanceof Error ? reason.message : 'Could not sign in.'); }
    finally { setBusy(false); }
  };
  type SiteField = Exclude<keyof SiteSettings, 'galleries' | 'activity'>;
  const updateField = (name: SiteField, value: string) => setSettings((s) => ({ ...s, [name]: value }));
  const saveSettings = async (event: FormEvent) => {
    event.preventDefault(); setBusy(true); setError(''); setNotice('');
    try {
      await api('/admin/site', { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(settings) });
      await refresh(token); window.dispatchEvent(new Event('photolio-site-updated')); setNotice('Website settings saved.');
    } catch (reason) { setError(reason instanceof Error ? reason.message : 'Could not save settings.'); }
    finally { setBusy(false); }
  };
  const saveGallery = async (event: FormEvent) => {
    event.preventDefault(); setBusy(true); setError(''); setNotice('');
    try {
      await api('/admin/galleries', { method: 'POST', headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ slug: gallerySlug, title: galleryTitle, description: galleryDescription }) });
      setGalleryTitle(''); setGallerySlug(''); setGalleryDescription(''); await refresh(token); setNotice('Gallery saved.');
    } catch (reason) { setError(reason instanceof Error ? reason.message : 'Could not save gallery.'); }
    finally { setBusy(false); }
  };
  const uploadPhotos = async (event: FormEvent) => {
    event.preventDefault();
    if (!files?.length || !uploadGallery) return;
    setBusy(true); setError(''); setNotice(''); let uploaded = 0;
    try {
      for (const file of Array.from(files)) {
        const query = '?gallery=' + encodeURIComponent(uploadGallery) + '&filename=' + encodeURIComponent(file.name);
        await api('/admin/upload' + query, { method: 'POST', headers: { 'Content-Type': 'application/octet-stream' }, body: file });
        uploaded++;
      }
      setFiles(null);
      const input = document.getElementById('photo-files') as HTMLInputElement | null;
      if (input) input.value = '';
      await refresh(token); setNotice(uploaded + ' photo(s) uploaded.');
    } catch (reason) {
      setError(uploaded + ' photo(s) uploaded. ' + (reason instanceof Error ? reason.message : 'Upload failed.'));
      await refresh(token);
    } finally { setBusy(false); }
  };
  const logout = () => { window.localStorage.removeItem('photolioAdminToken'); setToken(''); setDashboard(undefined); setSettings(emptySettings); };

  if (!token) return <main style={{ width: 'min(1000px, calc(100% - 32px))', margin: '0 auto 50px' }}>
    <h1>Website admin</h1><section style={box}><h2>Sign in</h2><p>Enter the admin key supplied by your website host.</p>
      <form onSubmit={login}><label style={field}>Admin key<input style={control} type="password" value={loginKey} onChange={(e) => setLoginKey(e.target.value)} required /></label>
        <button style={button} disabled={busy}>{busy ? 'Signing in…' : 'Sign in'}</button></form>
      {error && <p role="alert" style={{ color: '#d55' }}>{error}</p>}
    </section>
  </main>;

  const used = dashboard?.used_bytes || 0;
  const limit = dashboard?.limit_bytes || 100000000000;
  const percent = Math.min(100, limit ? used / limit * 100 : 0);
  return <main style={{ width: 'min(1100px, calc(100% - 32px))', margin: '0 auto 60px' }}>
    <div style={{ ...row, justifyContent: 'space-between', alignItems: 'center' }}><div><h1>Website admin</h1><p style={muted}>Manage your photography website and galleries.</p></div>
      <button style={{ ...button, background: '#666' }} onClick={logout}>Sign out</button></div>
    {error && <p role="alert" style={{ color: '#d55' }}>{error}</p>}
    {notice && <p role="status">{notice}</p>}
    <section style={box}><h2>Overview</h2><div style={row}>
      <div style={{ ...box, flex: 1 }}><strong style={{ fontSize: 26 }}>{dashboard?.photo_count || 0}</strong><div style={muted}>Photos</div></div>
      <div style={{ ...box, flex: 1 }}><strong style={{ fontSize: 26 }}>{dashboard?.gallery_count || 0}</strong><div style={muted}>Galleries</div></div>
      <div style={{ ...box, flex: 1 }}><strong style={{ fontSize: 26 }}>{prettyBytes(used)}</strong><div style={muted}>Storage used</div></div>
    </div>
    <p>{percent.toFixed(1)}% of {prettyBytes(limit)} used</p>
    <div style={{ height: 10, background: '#8885', borderRadius: 20, overflow: 'hidden' }}><div style={{ height: '100%', width: percent + '%', background: '#3d7569' }} /></div></section>

    <section style={box}><h2>Website details and appearance</h2><form onSubmit={saveSettings}>
      <div style={row}>
        <label style={field}>Website title<input style={control} value={settings.title} onChange={(e) => updateField('title', e.target.value)} required /></label>
        <label style={field}>Tagline<input style={control} value={settings.tagline} onChange={(e) => updateField('tagline', e.target.value)} /></label>
        <label style={field}>Theme<select style={control} value={settings.theme} onChange={(e) => updateField('theme', e.target.value)}>
          <option value="classic">Classic</option><option value="dark">Dark</option><option value="warm">Warm</option><option value="editorial">Editorial</option>
        </select></label>
      </div><div style={row}>
        <label style={field}>Contact email<input style={control} type="email" value={settings.contact_email} onChange={(e) => updateField('contact_email', e.target.value)} /></label>
        <label style={field}>Contact phone<input style={control} value={settings.contact_phone} onChange={(e) => updateField('contact_phone', e.target.value)} /></label>
      </div><div style={row}>
        <label style={field}>Website URL<input style={control} type="url" value={settings.website_url} onChange={(e) => updateField('website_url', e.target.value)} /></label>
        <label style={field}>Instagram URL<input style={control} type="url" value={settings.instagram_url} onChange={(e) => updateField('instagram_url', e.target.value)} /></label>
        <label style={field}>YouTube URL<input style={control} type="url" value={settings.youtube_url} onChange={(e) => updateField('youtube_url', e.target.value)} /></label>
      </div><div style={row}>
        <label style={field}>Facebook URL<input style={control} type="url" value={settings.facebook_url} onChange={(e) => updateField('facebook_url', e.target.value)} /></label>
        <label style={field}>TikTok URL<input style={control} type="url" value={settings.tiktok_url} onChange={(e) => updateField('tiktok_url', e.target.value)} /></label>
      </div>
      <label style={field}>About text<textarea style={{ ...control, minHeight: 110 }} value={settings.about} onChange={(e) => updateField('about', e.target.value)} /></label>
      <button style={button} disabled={busy}>Save website details</button>
    </form></section>

    <section style={box}><h2>Galleries</h2><p>Create separate collections such as portraits, wildlife, weddings or travel.</p>
      <div style={row}>{(settings.galleries || []).map((gallery: GalleryModel) => <div key={gallery.slug} style={{ ...box, flex: 1, minWidth: 180 }}>
        <strong>{gallery.title}</strong><div style={muted}>/{gallery.slug}</div><p>{gallery.description}</p></div>)}</div>
      <form onSubmit={saveGallery}><div style={row}>
        <label style={field}>Gallery title<input style={control} value={galleryTitle} onChange={(e) => {
          const value = e.target.value; setGalleryTitle(value);
          if (!gallerySlug) setGallerySlug(value.toLowerCase().trim().replace(/[^a-z0-9_-]+/g, '-').replace(/^-|-$/g, ''));
        }} required /></label>
        <label style={field}>URL slug<input style={control} value={gallerySlug} onChange={(e) => setGallerySlug(e.target.value.toLowerCase().trim().replace(/[^a-z0-9_-]+/g, '-').replace(/^-|-$/g, ''))} required /></label>
      </div><label style={field}>Description<textarea style={{ ...control, minHeight: 70 }} value={galleryDescription} onChange={(e) => setGalleryDescription(e.target.value)} /></label>
        <button style={button} disabled={busy}>Create / update gallery</button></form>
    </section>

    <section style={box}><h2>Upload photos</h2><form onSubmit={uploadPhotos}>
      <label style={field}>Destination gallery<select style={control} value={uploadGallery} onChange={(e) => setUploadGallery(e.target.value)} required>
        {(settings.galleries || []).map((gallery) => <option key={gallery.slug} value={gallery.slug}>{gallery.title}</option>)}
      </select></label>
      <label style={field}>Select image files<input id="photo-files" style={control} type="file" accept=".jpg,.jpeg,.png,.webp,.gif,.tiff" multiple onChange={(e) => setFiles(e.target.files)} required /></label>
      <p style={muted}>JPG, PNG, WebP, GIF and TIFF. Uploads count towards this site's shared 100 GB photo quota.</p>
      <button style={button} disabled={busy || !files?.length}>{busy ? 'Working…' : 'Upload ' + (files?.length || 0) + ' photo(s)'}</button>
    </form></section>

    <section style={box}><h2>Recent activity</h2>
      {(dashboard?.activity || []).length === 0 ? <p style={muted}>No activity yet.</p> :
        dashboard!.activity.map((item, index) => <div key={item.at + index} style={{ display: 'flex', flexWrap: 'wrap', justifyContent: 'space-between', gap: 12, borderBottom: '1px solid #8885', padding: '12px 0' }}>
          <div><strong>{item.action}</strong><div>{item.detail}</div></div><small style={muted}>{new Date(item.at).toLocaleString()}</small>
        </div>)}
    </section>
  </main>;
};