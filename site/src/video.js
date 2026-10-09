// The landing page's video: a YouTube player in a frame, which no element
// of the language makes. Its origin is `meta.frame` in the config. Not the
// privacy-enhanced youtube-nocookie.com: with no cookie to go on, YouTube
// asks the reader to sign in to prove they are not a bot.

/**
 * Put a YouTube player for one video into a node.
 * @param {HTMLElement} node
 * @param {string} id the video's id
 * @param {string} title what the video is, for a screen reader
 */
function embedYouTube(node, id, title) {
  const frame = document.createElement("iframe");
  frame.src = "https://www.youtube.com/embed/" + encodeURIComponent(id);
  frame.title = title;
  frame.loading = "lazy";
  frame.allow = "accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share";
  frame.referrerPolicy = "strict-origin-when-cross-origin";
  frame.allowFullscreen = true;
  node.replaceChildren(frame);
  return frame;
}
