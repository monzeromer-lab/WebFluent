use std::fs;
use std::path::Path;
use crate::error::Result;

pub fn run_init(name: &str, template: &str) -> Result<()> {
    let project_dir = Path::new(name);

    if project_dir.exists() {
        return Err(crate::error::WebFluentError::IoError(
            format!("Directory '{}' already exists", name)
        ));
    }

    match template {
        "spa" => generate_spa(name, project_dir)?,
        "static" => generate_static(name, project_dir)?,
        "pdf" => generate_pdf(name, project_dir)?,
        _ => {
            eprintln!("Unknown template '{}'. Use 'spa', 'static', or 'pdf'.", template);
            std::process::exit(1);
        }
    }

    println!("Created new WebFluent project: {} (template: {})", name, template);
    println!();
    println!("  cd {}", name);
    println!("  wf build");
    println!("  wf serve");

    Ok(())
}

// ═══════════════════════════════════════════════════════════
//  SPA Template — Interactive Dashboard
// ═══════════════════════════════════════════════════════════

fn generate_spa(name: &str, dir: &Path) -> Result<()> {
    fs::create_dir_all(dir.join("src/pages"))?;
    fs::create_dir_all(dir.join("src/components"))?;
    fs::create_dir_all(dir.join("src/stores"))?;
    fs::create_dir_all(dir.join("public"))?;

    // ── Config ──
    fs::write(dir.join("webfluent.app.json"), format!(r#"{{
  "name": "{}",
  "version": "1.0.0",
  "author": "",
  "theme": {{
    "name": "default",
    "mode": "light"
  }},
  "build": {{
    "output": "./build",
    "minify": true,
    "ssg": false
  }},
  "dev": {{
    "port": 3000
  }},
  "meta": {{
    "title": "{}",
    "description": "Built with WebFluent",
    "lang": "en"
  }}
}}"#, name, name))?;

    // ── App.wf ──
    fs::write(dir.join("src/App.wf"), format!(r#"App {{
    use AuthStore

    Navbar {{
        Navbar.Brand {{
            Text("{}", heading)
        }}
        Navbar.Links {{
            Link(to: "/") {{ Text("Dashboard") }}
            Link(to: "/tasks") {{ Text("Tasks") }}
            Link(to: "/settings") {{ Text("Settings") }}
        }}
        Navbar.Actions {{
            if AuthStore.isLoggedIn {{
                Avatar(initials: "U", primary)
                Link(to: "/profile") {{ Text("Profile") }}
                Button("Logout", small) {{ AuthStore.logout() }}
            }} else {{
                Button("Login", primary, small) {{ AuthStore.login("user@demo.com", "demo") }}
            }}
        }}
    }}

    Router {{
        Route(path: "/", page: Home)
        Route(path: "/tasks", page: Tasks)
        Route(path: "/settings", page: Settings)
        Route(path: "/profile", page: Profile)
    }}
}}"#, name))?;

    // ── Home.wf ──
    fs::write(dir.join("src/pages/Home.wf"), r#"Page Home (path: "/", title: "Dashboard") {
    use AuthStore
    use TaskStore

    Container(fadeIn) {
        Heading("Dashboard", h1, slideUp)
        Text("Welcome to your dashboard.", muted)

        Spacer()

        Row(gap: md) {
            Column(span: 4) {
                StatCard(title: "Total Tasks", value: TaskStore.tasks.length, color: "primary")
            }
            Column(span: 4) {
                StatCard(title: "Remaining", value: TaskStore.remaining, color: "warning")
            }
            Column(span: 4) {
                StatCard(title: "Completed", value: TaskStore.completed, color: "success")
            }
        }

        Spacer()

        Row(gap: md) {
            Column(span: 8) {
                Card {
                    Card.Header {
                        Heading("Recent Tasks", h2)
                    }
                    Card.Body {
                        if TaskStore.tasks.length == 0 {
                            Text("No tasks yet. Go to the Tasks page to add some!", muted, center)
                        } else {
                            for task in TaskStore.tasks {
                                Row(align: center, justify: between) {
                                    Row(align: center, gap: md) {
                                        Checkbox(checked: task.done, label: task.title)
                                    }
                                    Badge(task.priority, primary)
                                }
                                Divider()
                            }
                        }
                    }
                }
            }
            Column(span: 4) {
                Card(elevated) {
                    Card.Header {
                        Heading("Quick Actions", h2)
                    }
                    Card.Body {
                        Stack(gap: sm) {
                            Button("Add Task", primary, full) { navigate("/tasks") }
                            Button("Settings", full) { navigate("/settings") }
                            Button("View Profile", full) { navigate("/profile") }
                        }
                    }
                }
            }
        }
    }
}"#)?;

    // ── Tasks.wf ──
    fs::write(dir.join("src/pages/Tasks.wf"), r#"Page Tasks (path: "/tasks", title: "Tasks") {
    use TaskStore

    state newTask = ""
    state showDeleteModal = false
    state deleteId = 0

    Container(fadeIn) {
        Heading("My Tasks", h1)
        Text("{TaskStore.remaining} tasks remaining", muted)

        Spacer()

        Row(gap: md) {
            Input(text, bind: newTask, placeholder: "What needs to be done?", full) {
                on:keydown {
                    if key == "Enter" && newTask != "" {
                        TaskStore.add(newTask)
                        newTask = ""
                    }
                }
            }
            Button("Add", primary) {
                if newTask != "" {
                    TaskStore.add(newTask)
                    newTask = ""
                }
            }
        }

        Spacer()

        ButtonGroup {
            Button("All") { TaskStore.setFilter("all") }
            Button("Active") { TaskStore.setFilter("active") }
            Button("Done") { TaskStore.setFilter("done") }
        }

        Spacer()

        if TaskStore.filtered.length == 0 {
            Card(outlined, scaleIn) {
                Text("No tasks found.", muted, center)
            }
        } else {
            Stack(gap: sm) {
                for task in TaskStore.filtered, animate(slideUp, fadeOut, stagger: "50ms") {
                    TaskItem(
                        title: task.title,
                        done: task.done,
                        priority: task.priority
                    )
                }
            }
        }

        Modal(visible: showDeleteModal, title: "Delete Task") {
            Text("Are you sure you want to delete this task?")
            Modal.Footer {
                Button("Cancel") { showDeleteModal = false }
                Button("Delete", danger) {
                    TaskStore.remove(deleteId)
                    showDeleteModal = false
                }
            }
        }
    }
}"#)?;

    // ── Settings.wf ──
    fs::write(dir.join("src/pages/Settings.wf"), r#"Page Settings (path: "/settings", title: "Settings") {
    state username = "demo_user"
    state email = "user@example.com"
    state notifications = true
    state theme = "light"
    state fontSize = 16
    state language = "en"
    state autoSave = false
    state saved = false

    Container(fadeIn) {
        Heading("Settings", h1)

        Spacer()

        Tabs {
            TabPage("General") {
                Form {
                    Heading("Profile", h2)
                    Input(text, bind: username, label: "Username", placeholder: "Enter username")
                    Spacer(sm)
                    Input(email, bind: email, label: "Email", placeholder: "Enter email")
                    Spacer(sm)
                    Select(bind: language, label: "Language") {
                        Option("en", "English")
                        Option("es", "Spanish")
                        Option("ar", "Arabic")
                    }

                    Spacer()

                    Heading("Preferences", h2)
                    Switch(bind: notifications, label: "Email Notifications")
                    Spacer(sm)
                    Switch(bind: autoSave, label: "Auto-save changes")
                    Spacer(sm)
                    Slider(bind: fontSize, min: 12, max: 24, step: 1, label: "Font Size")

                    Spacer()

                    Radio(bind: theme, value: "light", label: "Light Theme")
                    Radio(bind: theme, value: "dark", label: "Dark Theme")

                    Spacer()

                    Button("Save Settings", primary) {
                        saved = true
                    }

                    show saved {
                        Spacer(sm)
                        Alert("Settings saved successfully!", success, dismissible)
                    }

                    on:submit {
                        saved = true
                    }
                }
            }
            TabPage("Account") {
                Card {
                    Card.Body {
                        Heading("Account Info", h2)
                        Text("Username: {username}")
                        Text("Email: {email}")
                        Text("Theme: {theme}")
                        Text("Font Size: {fontSize}px")
                        Spacer()
                        Text("Notifications: {notifications}", muted)
                        Text("Auto-save: {autoSave}", muted)
                    }
                }
            }
            TabPage("Danger Zone") {
                Card(outlined) {
                    Card.Body {
                        Heading("Danger Zone", h2, danger)
                        Text("These actions cannot be undone.", muted)
                        Spacer()
                        Button("Delete Account", danger) {
                            log("Delete account clicked")
                        }
                    }
                }
            }
        }
    }
}"#)?;

    // ── Profile.wf ──
    fs::write(dir.join("src/pages/Profile.wf"), r#"Page Profile (path: "/profile", title: "Profile") {
    use AuthStore

    state editing = false

    Container(fadeIn) {
        Heading("Profile", h1)

        Spacer()

        Row(gap: md) {
            Column(span: 4) {
                Card(elevated, scaleIn) {
                    Card.Body {
                        Stack(gap: md) {
                            Avatar(initials: "U", primary, large)
                            Heading("Demo User", h2, center)
                            Text("user@demo.com", muted, center)
                            Divider()
                            Badge("Active", success)
                            Spacer(sm)
                            Progress(value: 75, max: 100)
                            Text("Profile 75% complete", muted, small)
                        }
                    }
                }
            }
            Column(span: 8) {
                Card {
                    Card.Header {
                        Row(align: center, justify: between) {
                            Heading("Details", h2)
                            Button("Edit", primary, small) { editing = !editing }
                        }
                    }
                    Card.Body {
                        if editing {
                            Form {
                                Input(text, placeholder: "Full Name", label: "Name")
                                Spacer(sm)
                                Input(email, placeholder: "Email", label: "Email")
                                Spacer(sm)
                                Input(text, placeholder: "Bio", label: "Bio")
                                Spacer()
                                Row(gap: sm) {
                                    Button("Save", primary) { editing = false }
                                    Button("Cancel") { editing = false }
                                }

                                on:submit {
                                    editing = false
                                }
                            }
                        } else {
                            Stack(gap: md) {
                                Row {
                                    Text("Name:", bold)
                                    Text("Demo User")
                                }
                                Row {
                                    Text("Email:", bold)
                                    Text("user@demo.com")
                                }
                                Row {
                                    Text("Bio:", bold)
                                    Text("WebFluent developer")
                                }
                                Row {
                                    Text("Joined:", bold)
                                    Text("March 2026")
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}"#)?;

    // ── Components ──
    fs::write(dir.join("src/components/TaskItem.wf"), r#"Component TaskItem (title: String, done: Bool, priority: String) {
    Card(outlined) {
        Row(align: center, justify: between) {
            Row(align: center, gap: md) {
                Checkbox(checked: done, label: title)
            }
            Row(gap: sm) {
                Badge(priority, primary, small)
                IconButton(icon: "trash", danger, small, label: "Delete task")
            }
        }
    }
}"#)?;

    fs::write(dir.join("src/components/StatCard.wf"), r#"Component StatCard (title: String, value: Number, color: String) {
    Card(elevated, fadeIn) {
        Card.Body {
            Text(title, muted, small)
            Heading(value, h2)
        }
        style {
            borderLeft: "4px solid var(--color-primary)"
        }
    }
}"#)?;

    fs::write(dir.join("src/components/ThemeToggle.wf"), r#"Component ThemeToggle () {
    state isDark = false

    Switch(bind: isDark, label: "Dark Mode")
}"#)?;

    // ── Stores ──
    fs::write(dir.join("src/stores/tasks.wf"), r#"Store TaskStore {
    state tasks = [
        { id: 1, title: "Learn WebFluent", done: false, priority: "high" },
        { id: 2, title: "Build a dashboard", done: false, priority: "medium" },
        { id: 3, title: "Deploy to production", done: false, priority: "low" }
    ]
    state filter = "all"

    derived remaining = tasks.filter(t => !t.done).length
    derived completed = tasks.filter(t => t.done).length
    derived filtered = if filter == "all" {
        tasks
    } else if filter == "active" {
        tasks.filter(t => !t.done)
    } else {
        tasks.filter(t => t.done)
    }

    action add(title: String) {
        tasks.push({ id: tasks.length + 1, title: title, done: false, priority: "medium" })
    }

    action toggle(id: Number) {
        state task = tasks.filter(t => t.id == id)[0]
        task.done = !task.done
    }

    action remove(id: Number) {
        tasks = tasks.filter(t => t.id != id)
    }

    action setFilter(value: String) {
        filter = value
    }
}"#)?;

    fs::write(dir.join("src/stores/auth.wf"), r#"Store AuthStore {
    state user = null
    state authToken = ""

    derived isLoggedIn = user != null

    action login(email: String, password: String) {
        user = { name: "Demo User", email: email, role: "admin" }
        authToken = "demo-token-123"
    }

    action logout() {
        user = null
        authToken = ""
        navigate("/")
    }
}"#)?;

    Ok(())
}

// ═══════════════════════════════════════════════════════════
//  Static Site Template — Marketing/Blog with SSG + i18n
// ═══════════════════════════════════════════════════════════

fn generate_static(name: &str, dir: &Path) -> Result<()> {
    fs::create_dir_all(dir.join("src/pages"))?;
    fs::create_dir_all(dir.join("src/components"))?;
    fs::create_dir_all(dir.join("src/stores"))?;
    fs::create_dir_all(dir.join("src/translations"))?;
    fs::create_dir_all(dir.join("public"))?;

    // ── Config (SSG + i18n enabled) ──
    fs::write(dir.join("webfluent.app.json"), format!(r#"{{
  "name": "{}",
  "version": "1.0.0",
  "author": "",
  "theme": {{
    "name": "default",
    "mode": "light"
  }},
  "build": {{
    "output": "./build",
    "minify": true,
    "ssg": true
  }},
  "dev": {{
    "port": 3000
  }},
  "meta": {{
    "title": "{}",
    "description": "A modern website built with WebFluent",
    "lang": "en"
  }},
  "i18n": {{
    "defaultLocale": "en",
    "locales": ["en", "ar"],
    "dir": "src/translations"
  }}
}}"#, name, name))?;

    // ── Translations ──
    fs::write(dir.join("src/translations/en.json"), r#"{
    "nav.home": "Home",
    "nav.about": "About",
    "nav.blog": "Blog",
    "nav.contact": "Contact",
    "nav.language": "Language",

    "hero.title": "Build Beautiful Websites",
    "hero.subtitle": "A modern, fast, and accessible website built entirely with WebFluent — the web-first language for building SPAs and static sites.",
    "hero.cta": "Get Started",
    "hero.secondary": "Learn More",

    "features.title": "Why Choose Us",
    "features.subtitle": "Everything you need to succeed online.",
    "features.speed.title": "Lightning Fast",
    "features.speed.desc": "Static site generation delivers instant page loads with pre-rendered HTML.",
    "features.design.title": "Beautiful Design",
    "features.design.desc": "A built-in design system with tokens for colors, spacing, and typography.",
    "features.i18n.title": "Multi-Language",
    "features.i18n.desc": "Built-in internationalization with RTL support for global audiences.",
    "features.a11y.title": "Accessible",
    "features.a11y.desc": "Compile-time accessibility checks ensure your site works for everyone.",

    "about.title": "About Us",
    "about.mission.title": "Our Mission",
    "about.mission.text": "We believe the web should be fast, beautiful, and accessible to everyone. Our tools make it easy to build websites that meet these standards.",
    "about.team.title": "Our Team",
    "about.quote": "The best way to predict the future is to create it.",

    "blog.title": "Blog",
    "blog.subtitle": "Latest articles and updates.",
    "blog.read": "Read More",

    "contact.title": "Contact Us",
    "contact.subtitle": "We would love to hear from you.",
    "contact.name": "Your Name",
    "contact.email": "Email Address",
    "contact.subject": "Subject",
    "contact.subject.general": "General Inquiry",
    "contact.subject.support": "Support",
    "contact.subject.feedback": "Feedback",
    "contact.message": "Message",
    "contact.agree": "I agree to the privacy policy",
    "contact.send": "Send Message",
    "contact.success": "Thank you! Your message has been sent.",
    "contact.error": "Something went wrong. Please try again.",

    "footer.built": "Built with WebFluent",
    "footer.rights": "All rights reserved."
}"#)?;

    fs::write(dir.join("src/translations/ar.json"), r#"{
    "nav.home": "الرئيسية",
    "nav.about": "من نحن",
    "nav.blog": "المدونة",
    "nav.contact": "اتصل بنا",
    "nav.language": "اللغة",

    "hero.title": "ابنِ مواقع ويب جميلة",
    "hero.subtitle": "موقع حديث وسريع وسهل الوصول مبني بالكامل باستخدام ويب فلونت — لغة الويب الأولى لبناء تطبيقات الصفحة الواحدة والمواقع الثابتة.",
    "hero.cta": "ابدأ الآن",
    "hero.secondary": "اعرف المزيد",

    "features.title": "لماذا تختارنا",
    "features.subtitle": "كل ما تحتاجه للنجاح على الإنترنت.",
    "features.speed.title": "سريع كالبرق",
    "features.speed.desc": "توليد المواقع الثابتة يوفر تحميل فوري للصفحات مع HTML مُعد مسبقاً.",
    "features.design.title": "تصميم جميل",
    "features.design.desc": "نظام تصميم مدمج مع رموز للألوان والمسافات والخطوط.",
    "features.i18n.title": "متعدد اللغات",
    "features.i18n.desc": "دعم مدمج للتدويل مع دعم الكتابة من اليمين لليسار للجمهور العالمي.",
    "features.a11y.title": "سهل الوصول",
    "features.a11y.desc": "فحوصات إمكانية الوصول أثناء التجميع تضمن أن موقعك يعمل للجميع.",

    "about.title": "من نحن",
    "about.mission.title": "مهمتنا",
    "about.mission.text": "نؤمن بأن الويب يجب أن يكون سريعاً وجميلاً وسهل الوصول للجميع. أدواتنا تسهل بناء مواقع تلبي هذه المعايير.",
    "about.team.title": "فريقنا",
    "about.quote": "أفضل طريقة للتنبؤ بالمستقبل هي صنعه.",

    "blog.title": "المدونة",
    "blog.subtitle": "أحدث المقالات والتحديثات.",
    "blog.read": "اقرأ المزيد",

    "contact.title": "اتصل بنا",
    "contact.subtitle": "يسعدنا سماع رأيك.",
    "contact.name": "اسمك",
    "contact.email": "البريد الإلكتروني",
    "contact.subject": "الموضوع",
    "contact.subject.general": "استفسار عام",
    "contact.subject.support": "الدعم",
    "contact.subject.feedback": "ملاحظات",
    "contact.message": "الرسالة",
    "contact.agree": "أوافق على سياسة الخصوصية",
    "contact.send": "إرسال الرسالة",
    "contact.success": "شكراً لك! تم إرسال رسالتك.",
    "contact.error": "حدث خطأ ما. يرجى المحاولة مرة أخرى.",

    "footer.built": "مبني بواسطة ويب فلونت",
    "footer.rights": "جميع الحقوق محفوظة."
}"#)?;

    // ── App.wf ──
    fs::write(dir.join("src/App.wf"), format!(r#"App {{
    Navbar {{
        Navbar.Brand {{
            Text("{}", heading)
        }}
        Navbar.Links {{
            Link(to: "/") {{ Text(t("nav.home")) }}
            Link(to: "/about") {{ Text(t("nav.about")) }}
            Link(to: "/blog") {{ Text(t("nav.blog")) }}
            Link(to: "/contact") {{ Text(t("nav.contact")) }}
        }}
        Navbar.Actions {{
            Button("EN", small) {{ setLocale("en") }}
            Button("AR", small) {{ setLocale("ar") }}
        }}
    }}

    Router {{
        Route(path: "/", page: Home)
        Route(path: "/about", page: About)
        Route(path: "/blog", page: Blog)
        Route(path: "/contact", page: Contact)
    }}

    Footer
}}"#, name))?;

    // ── Home.wf ──
    fs::write(dir.join("src/pages/Home.wf"), r#"Page Home (path: "/", title: "Home") {
    Container {
        Spacer(xl)

        Stack(gap: md) {
            Heading(t("hero.title"), h1, center, slideUp)
            Text(t("hero.subtitle"), muted, center, fadeIn)
            Spacer(sm)
            Row(gap: md, justify: center) {
                Button(t("hero.cta"), primary, large) { navigate("/contact") }
                Button(t("hero.secondary"), large) { navigate("/about") }
            }
        }

        Spacer(xl)
        Divider()
        Spacer()

        Heading(t("features.title"), h2, center)
        Text(t("features.subtitle"), muted, center)

        Spacer()

        Grid(columns: 2, gap: lg) {
            FeatureCard(title: t("features.speed.title"), description: t("features.speed.desc"), icon: "zap")
            FeatureCard(title: t("features.design.title"), description: t("features.design.desc"), icon: "palette")
            FeatureCard(title: t("features.i18n.title"), description: t("features.i18n.desc"), icon: "globe")
            FeatureCard(title: t("features.a11y.title"), description: t("features.a11y.desc"), icon: "shield")
        }

        Spacer(xl)
    }
}"#)?;

    // ── About.wf ──
    fs::write(dir.join("src/pages/About.wf"), r#"Page About (path: "/about", title: "About") {
    Container(fadeIn) {
        Heading(t("about.title"), h1)

        Spacer()

        Row(gap: lg) {
            Column(span: 6) {
                Card(elevated) {
                    Card.Body {
                        Heading(t("about.mission.title"), h2)
                        Spacer(sm)
                        Text(t("about.mission.text"))
                        Spacer()
                        Blockquote(t("about.quote"))
                    }
                }
            }
            Column(span: 6) {
                Card {
                    Card.Body {
                        Heading(t("about.team.title"), h2)
                        Spacer(sm)
                        Stack(gap: md) {
                            TeamMember(name: "Monzer Omer", role: "Creator", initials: "MO")
                            Divider()
                            TeamMember(name: "Sara Ali", role: "Designer", initials: "SA")
                            Divider()
                            TeamMember(name: "Omar Hassan", role: "Developer", initials: "OH")
                        }
                    }
                }
            }
        }
    }
}"#)?;

    // ── Blog.wf ──
    fs::write(dir.join("src/pages/Blog.wf"), r#"Page Blog (path: "/blog", title: "Blog") {
    Container(fadeIn) {
        Heading(t("blog.title"), h1)
        Text(t("blog.subtitle"), muted)

        Spacer()

        Grid(columns: 3, gap: md) {
            Card(elevated, scaleIn) {
                Card.Body {
                    Badge("New", success)
                    Spacer(sm)
                    Heading("Getting Started with WebFluent", h2)
                    Text("Learn how to build your first web application using the WebFluent language.", muted)
                    Spacer(sm)
                    Tag("Tutorial")
                    Tag("Beginner")
                }
                Card.Footer {
                    Link(to: "/blog") { Text(t("blog.read"), primary) }
                }
            }
            Card(elevated, scaleIn) {
                Card.Body {
                    Badge("Popular", primary)
                    Spacer(sm)
                    Heading("Building with SSG", h2)
                    Text("How to use Static Site Generation for lightning-fast page loads.", muted)
                    Spacer(sm)
                    Tag("SSG")
                    Tag("Performance")
                }
                Card.Footer {
                    Link(to: "/blog") { Text(t("blog.read"), primary) }
                }
            }
            Card(elevated, scaleIn) {
                Card.Body {
                    Spacer(sm)
                    Heading("Multi-Language Sites", h2)
                    Text("Add internationalization to your WebFluent site with built-in i18n support.", muted)
                    Spacer(sm)
                    Tag("i18n")
                    Tag("RTL")
                }
                Card.Footer {
                    Link(to: "/blog") { Text(t("blog.read"), primary) }
                }
            }
        }
    }
}"#)?;

    // ── Contact.wf ──
    fs::write(dir.join("src/pages/Contact.wf"), r#"Page Contact (path: "/contact", title: "Contact") {
    state name = ""
    state email = ""
    state subject = "general"
    state message = ""
    state agreed = false
    state submitted = false
    state errorMsg = ""

    Container(fadeIn) {
        Heading(t("contact.title"), h1)
        Text(t("contact.subtitle"), muted)

        Spacer()

        Row(gap: lg) {
            Column(span: 8) {
                Card(elevated) {
                    Card.Body {
                        show submitted {
                            Alert(t("contact.success"), success)
                            Spacer()
                        }

                        Form {
                            Input(text, bind: name, label: t("contact.name"), placeholder: t("contact.name"), required: true)
                            Spacer(sm)
                            Input(email, bind: email, label: t("contact.email"), placeholder: t("contact.email"), required: true)
                            Spacer(sm)
                            Select(bind: subject, label: t("contact.subject")) {
                                Option("general", t("contact.subject.general"))
                                Option("support", t("contact.subject.support"))
                                Option("feedback", t("contact.subject.feedback"))
                            }
                            Spacer(sm)
                            Input(text, bind: message, label: t("contact.message"), placeholder: t("contact.message"))
                            Spacer()
                            Checkbox(bind: agreed, label: t("contact.agree"))
                            Spacer()
                            Button(t("contact.send"), primary, large, full) {
                                submitted = true
                            }

                            on:submit {
                                submitted = true
                            }
                        }
                    }
                }
            }
            Column(span: 4) {
                Card {
                    Card.Body {
                        Heading("Info", h2)
                        Spacer(sm)
                        Stack(gap: sm) {
                            Text("hello@example.com")
                            Text("+1 (555) 123-4567")
                            Text("123 Main Street")
                        }
                    }
                }
            }
        }
    }
}"#)?;

    // ── Components ──
    fs::write(dir.join("src/components/FeatureCard.wf"), r#"Component FeatureCard (title: String, description: String, icon: String) {
    Card(elevated, scaleIn) {
        Card.Body {
            Icon(icon, primary, large)
            Spacer(sm)
            Heading(title, h2)
            Text(description, muted)
        }
    }
}"#)?;

    fs::write(dir.join("src/components/TeamMember.wf"), r#"Component TeamMember (name: String, role: String, initials: String) {
    Row(align: center, gap: md) {
        Avatar(initials: initials, primary)
        Stack {
            Text(name, bold)
            Text(role, muted, small)
        }
    }
}"#)?;

    fs::write(dir.join("src/components/Footer.wf"), r#"Component Footer () {
    Divider()
    Container {
        Spacer()
        Row(align: center, justify: between) {
            Text(t("footer.built"), muted, small)
            Text(t("footer.rights"), muted, small)
        }
        Spacer()
    }
}"#)?;

    Ok(())
}

fn generate_pdf(name: &str, project_dir: &Path) -> Result<()> {
    fs::create_dir_all(project_dir.join("src/pages"))?;

    fs::write(project_dir.join("webfluent.app.json"), format!(r#"{{
  "name": "{name}",
  "version": "0.1.0",
  "build": {{
    "output": "./build",
    "output_type": "pdf",
    "pdf": {{
      "page_size": "A4",
      "margins": {{ "top": 72, "bottom": 72, "left": 72, "right": 72 }},
      "default_font": "Helvetica",
      "default_font_size": 12,
      "output_filename": "{name}.pdf"
    }}
  }}
}}"#))?;

    fs::write(project_dir.join("src/pages/Report.wf"), format!(r#"Page Report (path: "/", title: "{name} Report") {{
    Document(page_size: "A4") {{
        Header {{
            Text("{name}", muted, small, right)
        }}

        Footer {{
            Text("Confidential", muted, small, center)
        }}

        Section {{
            Heading("{name}", h1)

            Spacer(sm)

            Text("This document was generated with WebFluent's PDF output.")

            Spacer()

            Heading("Summary", h2)

            Paragraph {{
                Text("WebFluent generates PDF documents directly from .wf source files. The same declarative syntax used for web UIs works for documents.")
            }}

            Divider()

            Heading("Data Table", h2)

            Table {{
                Thead {{
                    Trow {{
                        Tcell("Item")
                        Tcell("Category")
                        Tcell("Status")
                        Tcell("Value")
                    }}
                }}
                Tbody {{
                    Trow {{
                        Tcell("Widget A")
                        Tcell("Hardware")
                        Tcell("Active")
                        Tcell("$1,200")
                    }}
                    Trow {{
                        Tcell("Service B")
                        Tcell("Software")
                        Tcell("Pending")
                        Tcell("$3,400")
                    }}
                    Trow {{
                        Tcell("License C")
                        Tcell("Legal")
                        Tcell("Complete")
                        Tcell("$800")
                    }}
                }}
            }}

            Spacer()

            Heading("Key Points", h3)

            List {{
                Text("PDF output uses raw PDF 1.7 — no external dependencies")
                Text("Tables, headings, text, lists, and code blocks are supported")
                Text("Automatic page breaks with headers and footers on every page")
                Text("Font support for all 14 PDF base fonts")
            }}

            Spacer()

            Alert("Interactive elements like Button and Input are rejected at compile time in PDF mode.", info)

            Spacer()

            Code("Document(page_size: \"A4\") \{{\n    Heading(\"Hello!\", h1)\n    Text(\"Generated with WebFluent.\")\n\}}", block)

            Spacer()

            Blockquote {{
                Text("WebFluent — build for the web, print for the world.")
            }}

            PageBreak()

            Heading("Page Two", h1)

            Paragraph {{
                Text("Content continues after PageBreak. Headers and footers repeat on every page.")
            }}

            Progress(value: 75, max: 100, primary)

            Spacer()

            Badge("Complete", success)
        }}
    }}
}}
"#))?;

    Ok(())
}
