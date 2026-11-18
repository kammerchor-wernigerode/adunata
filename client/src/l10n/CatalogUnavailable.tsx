/** Without a catalog no text can be translated, so this notice is the only text written into the client. */
export function CatalogUnavailable() {
  return (
    <main role="alert" className="p-6">
      <p lang="de">Adunata kann seine Texte gerade nicht laden. Bitte lade die Seite neu.</p>
      <p lang="en">Adunata cannot load its texts right now. Please reload the page.</p>
    </main>
  )
}
