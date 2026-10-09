// The landing page's video: a YouTube player in a frame, which no element
// of the language makes. The privacy-enhanced domain sets no cookie until
// the reader plays it; the origin is `meta.frame` in the config.

/**
 * Put a YouTube player for one video into a node.
 * @param {HTMLElement} node
 * @param {string} id the video's id
 * @param {string} title what the video is, for a screen reader
 */
function embedYouTube(node, id, title) {
  const frame = document.createElement("iframe");
  frame.src = "https://www.youtube-nocookie.com/embed/" + encodeURIComponent(id);
  frame.title = title;
  frame.loading = "lazy";
  frame.allow = "accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share";
  frame.referrerPolicy = "strict-origin-when-cross-origin";
  frame.allowFullscreen = true;
  node.replaceChildren(frame);
  return frame;
}
