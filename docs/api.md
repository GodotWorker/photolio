# API Documentation

## Public endpoints

- \`GET /site\` — public site details, theme, social links and gallery list.
- \`GET /images\` — list all photo metadata.
- \`GET /images?gallery=SLUG\` — list photos in one gallery.
- \`GET /images/{id}\` — image bytes.
- \`GET /images/{id}/meta\` — metadata.
- \`GET /images/{id}/thumbnail?width=640\` — scaled thumbnail.

## Admin endpoints

All admin endpoints require \`Authorization: Bearer YOUR_ADMIN_KEY\`.

- \`GET /admin/site\` — site settings, galleries and recent activity.
- \`PUT /admin/site\` — update site details, theme, contact information and social links.
- \`GET /admin/dashboard\` — photo/gallery counts, quota usage and recent activity.
- \`POST /admin/galleries\` — create or update a gallery. JSON body: \`{"slug":"wildlife","title":"Wildlife","description":"Animals and nature"}\`.
- \`POST /admin/upload?gallery=wildlife&filename=photo.jpg\` — stream one image as the raw request body. Repeat for multiple files.

The upload endpoint restricts file extensions and checks the configured total storage quota before and during file streaming.

## Image metadata

\`\`\`ts
type ImageModel = {
  id: string;
  name: string;
  group?: string;
  timestamp: string;
  blurhash: BlurHash;
  dimensions: { width: number; height: number };
  ratio: number;
  exif?: ExifModel;
};
\`\`\`
