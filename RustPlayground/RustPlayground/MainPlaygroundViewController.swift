//
//  MainPlaygroundVewController.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-04-15.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Cocoa

let OUTPUT_TOOLBAR_ITEM_TAG = 10;
let TOOLCHAIN_SELECT_TOOLBAR_ITEM_TAG = 13;
let RUN_TOOLBAR_ITEM_TAG = 14;
let ACTIVITY_SPINNER_TOOLBAR_ITEM_TAG = -1;
let SHARE_DISABLED_TOOLBAR_ITEM_TAG = 15;
let SHARE_TOOLBAR_ITEM_TAG = 16;

let SHARE_TOOLBAR_IDENTIFIER = NSToolbarItem.Identifier(rawValue: "share")
let SHARE_DISABLED_TOOLBAR_IDENTIFIER = NSToolbarItem.Identifier(rawValue: "shareDisabled")

let TOOLCHAIN_ITEM_TAG_OFFSET = 1000;

let SHARE_PREFERENCES_TAB_VIEW_INDEX = 1;

class MainPlaygroundViewController: NSSplitViewController {

    var outputViewController: OutputViewController {
        return splitViewItems[1].viewController as! OutputViewController
    }

    var editViewController: EditViewController {
        return splitViewItems[0].viewController as! EditViewController
    }

    lazy var toggleOutputToolbarButton: NSButton = {
        let toolbarItem = view.window?.toolbar?.items.first {
            $0.tag == OUTPUT_TOOLBAR_ITEM_TAG
        }
        return toolbarItem!.view as! NSButton
    }()

    lazy var toolchainSelectButton: NSPopUpButton = {
        let toolbarItem = view.window?.toolbar?.items.first {
            $0.tag == TOOLCHAIN_SELECT_TOOLBAR_ITEM_TAG
        }
        return toolbarItem!.view as! NSPopUpButton
    }()

    lazy var runButton: NSButton = {
        let toolbarItem = view.window?.toolbar?.items.first {
            $0.tag == RUN_TOOLBAR_ITEM_TAG
        }
        return toolbarItem!.view as! NSButton
    }()

    lazy var activitySpinner: NSProgressIndicator = {
        let toolbarItem = view.window?.toolbar?.items.first {
            $0.tag == ACTIVITY_SPINNER_TOOLBAR_ITEM_TAG
        }
        return toolbarItem!.view as! NSProgressIndicator
    }()

    override func viewDidLoad() {
        super.viewDidLoad()
        NotificationCenter.default.addObserver(self,
                                               selector: #selector(toolchainsChanged(_:)),
                                               name: AppDelegate.toolchainsChangedNotification,
                                               object: nil)

        NotificationCenter.default.addObserver(self,
                                               selector: #selector(githubTokenChanged(_:)),
                                               name: EditorPreferences.githubTokenChangedNotification,
                                               object: nil)
    }

    override func viewDidAppear() {
        super.viewDidAppear()
        let initSplitHeight = max(200, view.frame.height / 3).rounded(.down);
        splitView.setPosition(view.frame.height - initSplitHeight, ofDividerAt: 0)
        splitViewItems[1].isCollapsed = true
        activitySpinner.isDisplayedWhenStopped = false
        configureShareToolbarItem()
    }

    func configureShareToolbarItem() {
        guard let toolbar = view.window?.toolbar else { return }
        if let toRemove = toolbar.items.firstIndex(where: {
            $0.tag == SHARE_TOOLBAR_ITEM_TAG
            || $0.tag == SHARE_DISABLED_TOOLBAR_ITEM_TAG
        }) {
            toolbar.removeItem(at: toRemove)
        }
        let shareDisabled = EditorPreferences.shared.githubToken.isEmpty
        let identifier = shareDisabled ? SHARE_DISABLED_TOOLBAR_IDENTIFIER : SHARE_TOOLBAR_IDENTIFIER
        toolbar.insertItem(withItemIdentifier: identifier, at: toolbar.items.count)
    }

    @objc func toolchainsChanged(_ notification: Notification) {
        toolchainSelectButton.removeAllItems()
        for toolchain in AppDelegate.shared.toolchains {
            toolchainSelectButton.addItem(withTitle: toolchain.displayName)
        }

        if AppDelegate.shared.toolchains.count == 0 {
            //TODO: only show this if rustup is actually missing?
            showMissingRustupView()
        }
        toolchainSelectButton.isEnabled = AppDelegate.shared.toolchains.count > 1
        toolchainSelectButton.isHidden = AppDelegate.shared.toolchains.count == 0
        runButton.isEnabled = AppDelegate.shared.toolchains.count > 0

        //TODO: show a warning if no toolchains are found
    }

    @objc func githubTokenChanged(_ notification: Notification) {
        configureShareToolbarItem()
    }

    @IBAction func debugPrintEnvironment(_ sender: Any?) {
        let env = ProcessInfo.processInfo.environment
        if !outputViewIsVisible {
            outputViewIsVisible = true
        }

        outputViewController.clearOutput()
        outputViewController.printHeader("ENV")

        for (key, value) in env {
            outputViewController.printText(("\(key):\n\t\(value)\n"))
        }
        outputViewController.printHeader("Done")
    }

    func showMissingRustupView() {
        let rustupView = MissingRustupInfoView(frame: self.view.bounds)
        rustupView.translatesAutoresizingMaskIntoConstraints = false
        self.view.addSubview(rustupView)
        self.view.addConstraints([
            NSLayoutConstraint(item: self.view, attribute: .leading, relatedBy: .equal, toItem: rustupView, attribute: .leading, multiplier: 1.0, constant: 0),
            NSLayoutConstraint(item: self.view, attribute: .trailing, relatedBy: .equal, toItem: rustupView, attribute: .trailing, multiplier: 1.0, constant: 0),
            NSLayoutConstraint(item: self.view, attribute: .top, relatedBy: .equal, toItem: rustupView, attribute: .top, multiplier: 1.0, constant: 0),
            NSLayoutConstraint(item: self.view, attribute: .bottom, relatedBy: .equal, toItem: rustupView, attribute: .bottom, multiplier: 1.0, constant: 0),
            ])
        self.view.needsLayout = true
    }

    var outputViewIsVisible: Bool = false {
        didSet {
            let isVisible = outputViewIsVisible
            toggleOutputToolbarButton.highlight(true)
            NSAnimationContext.runAnimationGroup({ (context) in
                context.allowsImplicitAnimation = true
                context.duration = 0.25
                self.splitViewItems[1].isCollapsed = !isVisible
                self.splitView.layoutSubtreeIfNeeded()
            }, completionHandler: {
                let newState: NSControl.StateValue = isVisible ? .on : .off
                self.toggleOutputToolbarButton.highlight(false)
                self.toggleOutputToolbarButton.state = newState
                if newState == .on {
                    AppDelegate.shared.toggleConsoleMenu.title = "Hide Console"
                } else {
                    AppDelegate.shared.toggleConsoleMenu.title = "Show Console"
                }
            })
        }
    }

    @IBAction func increaseFontSize(_ sender: Any?) {
        EditorPreferences.shared.increaseFontSize()
    }

    @IBAction func decreaseFontSize(_ sender: Any?) {
        EditorPreferences.shared.decreaseFontSize()
    }

    @IBAction func toggleOutputView(_ sender: NSButton?) {
        outputViewIsVisible = !outputViewIsVisible
    }

    @IBAction func toolchainSelectAction(_ sender: NSToolbarItem) {

    }

    @IBAction func toggleConsoleAction(_ sender: NSMenuItem) {
        outputViewIsVisible = !outputViewIsVisible
    }

    @IBAction func shareDisabledAction(_ sender: NSToolbarItem) {
        let prefController = AppDelegate.shared.preferencesWindowController
        guard let tabController = prefController.contentViewController as? NSTabViewController else { return }
        tabController.selectedTabViewItemIndex = SHARE_PREFERENCES_TAB_VIEW_INDEX;
        prefController.showWindow(nil)
    }

    @IBAction func createGist(_ sender: Any?) {
        createGistAndReportErrors() { NSWorkspace.shared.open($0.toGistUrl()) }
    }

    @IBAction func sendToWebPlayground(_ sender: Any?) {
        createGistAndReportErrors() { NSWorkspace.shared.open($0.toPlaygroundUrl()) }
    }

    func createGistAndReportErrors(completion: @escaping (GistIdentifier) -> ()) {
        let text = AppDelegate.shared.core.getDocument()
        GithubConnection(token: EditorPreferences.shared.githubToken) .createGist(withContent: text) { [weak self] (result) in
            DispatchQueue.main.async {
                switch result {
                case .failure(let err):
                    self?.showGithubError(err)
                case .success(let gistId):
                    completion(gistId)
                }
            }
        }
    }

    func showGithubError(_ error: GithubError) {
        guard let window = view.window else { return }

        let alert = NSAlert(error: error)
        alert.messageText = error.localizedDescription
        alert.beginSheetModal(for: window)
    }

    @IBAction func buildAction(_ sender: Any?) {
        build(andRun: false)
    }

    @IBAction func runAction(_ sender: Any?) {
        build(andRun: true)
    }

    func build(andRun run: Bool) {
        if !outputViewIsVisible {
            outputViewIsVisible = true
        }
        outputViewController.clearOutput()
        outputViewController.printInfo(text: "Compiling")

        activitySpinner.startAnimation(self)
        runButton.isEnabled = false

        let task = generateTask()
        let buildDir = AppDelegate.shared.defaultBuildDirectory
        DispatchQueue.global(qos: .default).async {
            let result = self.executeTask(task, inDirectory: buildDir)
            DispatchQueue.main.async {
                self.taskFinished(result, run: run)
            }
        }

    }

    func taskFinished(_ result: Result<CompilerResult, PlaygroundError>, run: Bool) {
        activitySpinner.stopAnimation(self)
        runButton.isEnabled = true
        switch result {
        case .failure(let badNews):
            outputViewController.printInfo(text: "Error")
            outputViewController.handleStdErr(text: badNews.message)
        case .success(let goodNews):
            displayTaskOutput(goodNews)
            if goodNews.success && run, let executablePath = goodNews.executable {
                runCommand(atPath: executablePath, handler: outputViewController)
            }
        }
        outputViewController.printInfo(text: "Done")
    }

    func displayTaskOutput(_ result: CompilerResult) {
        if result.stdErr.count > 0 {
            outputViewController.printHeader("Standard Error")
            outputViewController.printText(result.stdErr)
        }

        if result.stdOut.count > 0 {
            outputViewController.printHeader("Standard Output")
            outputViewController.printText(result.stdOut)
        }
    }

    func executeTask(_ task: CompilerTask, inDirectory directory: URL) -> Result<CompilerResult, PlaygroundError> {
        return RustPlayground.executeTask(inDirectory: directory, task: task, stderr: { [weak self] (line) in
            DispatchQueue.main.async {
                self?.outputViewController.handleRawStdErrLine(line)
            }
        })
    }

    func generateTask() -> CompilerTask {
        let activeToolchainIdx = toolchainSelectButton.indexOfSelectedItem
        let toolchain = AppDelegate.shared.toolchains[activeToolchainIdx].name
        let code = AppDelegate.shared.core.getDocument()
        let taskType: CompilerTask.TaskType = .run
        return CompilerTask(toolchain: toolchain, code: code, type: taskType, backtrace: true, release: false)
    }
}
