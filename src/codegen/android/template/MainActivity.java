package @@PACKAGE@@;

import android.app.Activity;
import android.content.ActivityNotFoundException;
import android.content.ClipData;
import android.content.Intent;
import android.content.pm.ApplicationInfo;
import android.graphics.Insets;
import android.net.Uri;
import android.os.Build;
import android.os.Bundle;
import android.view.ViewGroup;
import android.view.WindowInsets;
import android.webkit.MimeTypeMap;
import android.webkit.ValueCallback;
import android.webkit.WebChromeClient;
import android.webkit.WebResourceRequest;
import android.webkit.WebResourceResponse;
import android.webkit.WebView;
import android.webkit.WebViewClient;
import android.widget.FrameLayout;
import android.window.OnBackInvokedCallback;
import android.window.OnBackInvokedDispatcher;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.util.Collections;
import java.util.Locale;

/**
 * The app: one WebView showing the WebFluent build, which is in
 * {@code assets/www}.
 *
 * <p>{@code wf build} wrote this file once and will not write it again, so it
 * is yours to change. What it writes on every build is the web build, the
 * launcher icons, the {@code webfluent.xml} resources and
 * {@code app/webfluent.properties}.
 */
public class MainActivity extends Activity {
    /**
     * Where the build is served from. Android keeps this host for an app's
     * own files, so no request for it reaches the network, and the pages get
     * a real {@code https} origin: storage, {@code fetch} and the
     * Content-Security-Policy behave as they do on the web.
     */
    static final String HOST = "appassets.androidplatform.net";

    private static final int CHOOSE_FILES = 1;

    private WebView web;
    private ValueCallback<Uri[]> chosen;
    private OnBackInvokedCallback back;
    private boolean backHeld;

    @Override
    protected void onCreate(Bundle state) {
        super.onCreate(state);
        // A debug build can be inspected from desktop Chrome, at chrome://inspect.
        WebView.setWebContentsDebuggingEnabled(
                (getApplicationInfo().flags & ApplicationInfo.FLAG_DEBUGGABLE) != 0);

        int background = getColor(R.color.wf_background);
        FrameLayout root = new FrameLayout(this);
        root.setBackgroundColor(background);
        web = new WebView(this);
        web.setBackgroundColor(background);
        root.addView(web, new FrameLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        setContentView(root);
        keepClearOfSystemBars(root);

        web.getSettings().setJavaScriptEnabled(true);
        // localStorage: what `persist` keeps, and the reader's choice of theme.
        web.getSettings().setDomStorageEnabled(true);
        web.getSettings().setAllowFileAccess(false);
        web.setWebViewClient(new Pages());
        web.setWebChromeClient(new Chrome());

        if (Build.VERSION.SDK_INT >= 33) {
            back = () -> web.goBack();
        }
        if (state == null || web.restoreState(state) == null) {
            web.loadUrl("https://" + HOST + "/");
        }
    }

    @Override
    protected void onSaveInstanceState(Bundle out) {
        super.onSaveInstanceState(out);
        web.saveState(out);
    }

    @Override
    protected void onResume() {
        super.onResume();
        web.onResume();
    }

    @Override
    protected void onPause() {
        web.onPause();
        super.onPause();
    }

    @Override
    protected void onDestroy() {
        web.destroy();
        super.onDestroy();
    }

    /** Back goes back through the pages the reader visited, then leaves. */
    @Override
    @SuppressWarnings("deprecation")
    public void onBackPressed() {
        if (web.canGoBack()) {
            web.goBack();
        } else {
            super.onBackPressed();
        }
    }

    /**
     * From Android 13 the system asks ahead of time whether the app wants a
     * back gesture, so it can preview leaving: the app holds it only while
     * there is a page to go back to.
     */
    private void holdBack() {
        if (Build.VERSION.SDK_INT < 33) {
            return;
        }
        boolean hold = web.canGoBack();
        if (hold == backHeld) {
            return;
        }
        OnBackInvokedDispatcher dispatcher = getOnBackInvokedDispatcher();
        if (hold) {
            dispatcher.registerOnBackInvokedCallback(
                    OnBackInvokedDispatcher.PRIORITY_DEFAULT, back);
        } else {
            dispatcher.unregisterOnBackInvokedCallback(back);
        }
        backHeld = hold;
    }

    /**
     * From Android 15 an app that targets it draws behind the status and
     * navigation bars. The page is kept clear of them, of a camera cutout and
     * of the keyboard; the strips they cover show the page's background
     * colour.
     */
    private void keepClearOfSystemBars(FrameLayout root) {
        if (Build.VERSION.SDK_INT < 35 || getApplicationInfo().targetSdkVersion < 35) {
            return;
        }
        root.setOnApplyWindowInsetsListener((view, insets) -> {
            Insets clear = insets.getInsets(WindowInsets.Type.systemBars()
                    | WindowInsets.Type.displayCutout() | WindowInsets.Type.ime());
            view.setPadding(clear.left, clear.top, clear.right, clear.bottom);
            return WindowInsets.CONSUMED;
        });
    }

    /**
     * A file the build wrote; a page a static build rendered; or the app's
     * shell, which routes on its own — as a host serves a WebFluent site.
     */
    private WebResourceResponse serve(String path) {
        String name = path == null ? "" : path.replaceAll("^/+|/+$", "");
        if (name.contains("..")) {
            return missing();
        }
        String last = name.substring(name.lastIndexOf('/') + 1);
        String[] candidates = last.contains(".")
                ? new String[] {name}
                : new String[] {name.isEmpty() ? "index.html" : name + "/index.html", "index.html"};
        for (String file : candidates) {
            try {
                InputStream body = getAssets().open("www/" + file);
                String type = typeOf(file);
                return new WebResourceResponse(type, type.startsWith("text/") ? "utf-8" : null, body);
            } catch (IOException notThere) {
                // The next candidate.
            }
        }
        return missing();
    }

    private static WebResourceResponse missing() {
        return new WebResourceResponse("text/plain", "utf-8", 404, "Not Found",
                Collections.emptyMap(), new ByteArrayInputStream(new byte[0]));
    }

    private static String typeOf(String file) {
        String extension = file.substring(file.lastIndexOf('.') + 1).toLowerCase(Locale.ROOT);
        switch (extension) {
            case "html":
                return "text/html";
            case "js":
            case "mjs":
                return "text/javascript";
            case "css":
                return "text/css";
            case "json":
            case "map":
                return "application/json";
            case "webmanifest":
                return "application/manifest+json";
            case "svg":
                return "image/svg+xml";
            case "webp":
                return "image/webp";
            case "avif":
                return "image/avif";
            case "woff":
                return "font/woff";
            case "woff2":
                return "font/woff2";
            case "wasm":
                return "application/wasm";
            case "txt":
                return "text/plain";
            case "vtt":
                return "text/vtt";
            default:
                String type = MimeTypeMap.getSingleton().getMimeTypeFromExtension(extension);
                return type != null ? type : "application/octet-stream";
        }
    }

    /** The schemes a link may leave the app by — those a WebFluent page may link to. */
    private static boolean leaves(String scheme) {
        if (scheme == null) {
            return false;
        }
        switch (scheme) {
            case "http":
            case "https":
            case "mailto":
            case "tel":
            case "sms":
                return true;
            default:
                return false;
        }
    }

    /** What the pages load: the build's own files from the app, the rest from the network. */
    private final class Pages extends WebViewClient {
        @Override
        public WebResourceResponse shouldInterceptRequest(WebView view, WebResourceRequest request) {
            Uri url = request.getUrl();
            return HOST.equals(url.getHost()) ? serve(url.getPath()) : null;
        }

        @Override
        public boolean shouldOverrideUrlLoading(WebView view, WebResourceRequest request) {
            Uri url = request.getUrl();
            if (HOST.equals(url.getHost()) || !request.isForMainFrame()) {
                return false;
            }
            // A link out of the app opens where the phone opens it: a site in
            // the browser, `mailto:` in the mail app, `tel:` in the dialler.
            if (leaves(url.getScheme())) {
                try {
                    startActivity(new Intent(Intent.ACTION_VIEW, url));
                } catch (ActivityNotFoundException nothingOpensIt) {
                    // Nothing on this phone opens it; the page stays where it is.
                }
            }
            return true;
        }

        @Override
        public void doUpdateVisitedHistory(WebView view, String url, boolean isReload) {
            holdBack();
        }
    }

    /** What a page asks of the app around it: the files an upload field picks. */
    private final class Chrome extends WebChromeClient {
        @Override
        public boolean onShowFileChooser(
                WebView view, ValueCallback<Uri[]> callback, FileChooserParams params) {
            if (chosen != null) {
                chosen.onReceiveValue(null);
            }
            chosen = callback;
            Intent pick = params.createIntent();
            if (params.getMode() == FileChooserParams.MODE_OPEN_MULTIPLE) {
                pick.putExtra(Intent.EXTRA_ALLOW_MULTIPLE, true);
            }
            try {
                startActivityForResult(pick, CHOOSE_FILES);
                return true;
            } catch (ActivityNotFoundException nothingPicks) {
                chosen = null;
                return false;
            }
        }
    }

    @Override
    protected void onActivityResult(int request, int result, Intent data) {
        if (request != CHOOSE_FILES || chosen == null) {
            super.onActivityResult(request, result, data);
            return;
        }
        Uri[] files = null;
        if (result == RESULT_OK && data != null) {
            ClipData several = data.getClipData();
            if (several != null) {
                files = new Uri[several.getItemCount()];
                for (int i = 0; i < files.length; i++) {
                    files[i] = several.getItemAt(i).getUri();
                }
            } else if (data.getData() != null) {
                files = new Uri[] {data.getData()};
            }
        }
        chosen.onReceiveValue(files);
        chosen = null;
    }
}
