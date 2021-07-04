pub fn get_index() -> String {
    let template = format!(
        "
    <!DOCTYPE html>
<html>
  <head>
    <meta charset='utf-8' />
    <meta name='viewport' content='width=device-width' />
    <title>Cache buster Actix Web example</title>
  </head>
  <body>
    <img
      class='greetings'
      src='{}'
      alt='logo image'
    />
    <h1>
      Cache Buster
    </h1>
    <h2>
      May your cache long live and prosper!
    </h2>

    <h3>What is cache busting?</h3>

    <p>
      To optimise network load time, browsers cache static files. Caching
      greatly improves performance but how do you inform browsers to invalidate
      cache when your files have changed?
    </p>
    <p>
      Cache busting is a simple but effective solution for this issue. There are
      several ways to achieve this but the way this library does this is by
      changing file names to include the hash of the files' contents. So if you
      have <code>bundle.js</code>, it will become
      <code>bundle.long-sha256-hash.js</code>. This lets you set a super long
      cache age as, because of the file names changing, the path to the
      filename, too, will change. So as far as the browser is concerned, you are
      trying to load a file that it doesn't have. 
    </p>
    <p>
      Pretty neat, isn't it?
    </p>

    <h3>Features</h3>
    <ul>
      <li><code>SHA-256</code> based name generation during compile-time</li>
      <li>Processes files based on provided <code>MIME</code> filters</li>
      <li>
        Exposes modified names to program during runtime
      </li>
      <li>
        Route prefixes(optional)
      </li>
    </ul>
  </body>
  <link rel='stylesheet' href='{}' type='text/css' media='all'>
</html>",
        crate::FILES
            .get_full_path("./static/cachable/img/Spock_vulcan-salute.png")
            .unwrap(),
        crate::FILES
            .get_full_path("./static/cachable/css/main.css")
            .unwrap()
    );
    template
}
