//
//  PreferencesViewController.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-04-20.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Cocoa

fileprivate let MIN_TAB_WIDTH = 1;
fileprivate let MAX_TAB_WIDTH = 40;
fileprivate let SPACES_MENU_ITEM_TAG = 10;
fileprivate let TAB_MENU_ITEM_TAG = 20;

class EditorPreferences {
    static let editorFontChangedNotification = Notification.Name(rawValue: "net.cmyr.rust-playground.editorFontChanged")

    static let consoleFontChangedNotification = Notification.Name(rawValue: "net.cmyr.rust-playground.consoleFontChanged")

    enum Keys: String {
        case translateTabsToSpaces = "translate_tabs_to_spaces"
        case tabWidth = "tab_size"
        case autoIndent = "auto_indent"
        case enableLineBreaking = "word_wrap"
        case editorFontFace = "font_face"
        case editorFontSize = "font_size"
        case consoleFontFace = "console_font_face"
        case consoleFontSize = "console_font_size"
        case githubToken = "githubToken"
    }

    enum IndentType {
        case spaces, tabs
    }

    static let shared = EditorPreferences()

    static func registerDefaults() {
        let defaultFontSize: CGFloat = 14.0
        let defaultFont = NSFont.userFixedPitchFont(ofSize: defaultFontSize) ?? NSFont.systemFont(ofSize: defaultFontSize)

        UserDefaults.standard.register(defaults: [
            Keys.translateTabsToSpaces.rawValue: true,
            Keys.tabWidth.rawValue: 4,
            Keys.autoIndent.rawValue: true,
            Keys.enableLineBreaking.rawValue: true,
            Keys.editorFontFace.rawValue: defaultFont.fontName,
            Keys.editorFontSize.rawValue: defaultFontSize,
            Keys.consoleFontFace.rawValue: defaultFont.fontName,
            Keys.consoleFontSize.rawValue: defaultFontSize,
            Keys.githubToken.rawValue: "",
        ])
    }

    init() {
        EditorPreferences.registerDefaults()
        let defaults = UserDefaults.standard

        self.tabWidth = defaults.integer(forKey: Keys.tabWidth.rawValue)
        self.autoIndentEnabled = defaults.bool(forKey: Keys.autoIndent.rawValue)
        self.lineWrappingEnabled = defaults.bool(forKey: Keys.enableLineBreaking.rawValue)

        self.indentWithSpaces = defaults.bool(forKey: Keys.translateTabsToSpaces.rawValue)

        let editorFont = defaults.string(forKey: Keys.editorFontFace.rawValue) ?? ""
        let editorFontSize = CGFloat(defaults.double(forKey: Keys.editorFontSize.rawValue))

        let consoleFont = defaults.string(forKey: Keys.consoleFontFace.rawValue) ?? ""
        let consoleFontSize = CGFloat(defaults.double(forKey: Keys.consoleFontSize.rawValue))

        self.editorFont = NSFont(name: editorFont, size: editorFontSize) ?? NSFont.systemFont(ofSize: editorFontSize)
        self.consoleFont = NSFont(name: consoleFont, size: consoleFontSize) ?? NSFont.systemFont(ofSize: consoleFontSize)
        self.githubToken = defaults.string(forKey: Keys.githubToken.rawValue) ?? ""
    }

    func syncAllWithCore() {
        core.updateConfig([
            Keys.autoIndent.rawValue: autoIndentEnabled,
            Keys.editorFontFace.rawValue: editorFont.fontName,
            Keys.editorFontSize.rawValue: editorFont.pointSize,
            Keys.enableLineBreaking.rawValue: lineWrappingEnabled,
            Keys.tabWidth.rawValue: tabWidth,
            Keys.translateTabsToSpaces.rawValue: indentWithSpaces,
            ])
    }

    private var core: XiCoreProxy {
        return AppDelegate.shared.core
    }

    fileprivate(set) var tabWidth: Int {
        didSet {
            UserDefaults.standard.set(tabWidth, forKey: Keys.tabWidth.rawValue)
            core.updateConfig([Keys.tabWidth.rawValue: tabWidth])
        }
    }

    fileprivate(set) var indentWithSpaces: Bool {
        didSet {
            UserDefaults.standard.set(indentWithSpaces, forKey: Keys.translateTabsToSpaces.rawValue)
            core.updateConfig([Keys.translateTabsToSpaces.rawValue: indentWithSpaces])
        }
    }

    fileprivate(set) var autoIndentEnabled: Bool {
        didSet {
            UserDefaults.standard.set(autoIndentEnabled, forKey: Keys.autoIndent.rawValue)
            core.updateConfig([Keys.autoIndent.rawValue: autoIndentEnabled])
        }
    }

    fileprivate(set) var lineWrappingEnabled: Bool {
        didSet {
            UserDefaults.standard.set(lineWrappingEnabled, forKey: Keys.enableLineBreaking.rawValue)
            core.updateConfig([Keys.enableLineBreaking.rawValue: lineWrappingEnabled])
        }
    }

    fileprivate(set) var editorFont: NSFont {
        didSet {
            UserDefaults.standard.set(editorFont.fontName, forKey: Keys.editorFontFace.rawValue)
            UserDefaults.standard.set(editorFont.pointSize, forKey: Keys.editorFontSize.rawValue)
            core.updateConfig([Keys.editorFontFace.rawValue: editorFont.fontName])
            core.updateConfig([Keys.editorFontSize.rawValue: editorFont.pointSize])
            NotificationCenter.default.post(name: EditorPreferences.editorFontChangedNotification, object: nil)
        }
    }

    fileprivate(set) var consoleFont: NSFont {
        didSet {
            UserDefaults.standard.set(consoleFont.fontName, forKey: Keys.consoleFontFace.rawValue)
            UserDefaults.standard.set(consoleFont.pointSize, forKey: Keys.consoleFontSize.rawValue)

            NotificationCenter.default.post(name: EditorPreferences.consoleFontChangedNotification, object: nil)
            // NOTE: core doesn't know about the console
        }
    }

    internal var githubToken: String {
        didSet {
            UserDefaults.standard.set(githubToken, forKey: Keys.githubToken.rawValue)
        }
    }

    func increaseFontSize() {
        let editorSizeStep = sizeAdjustmentUnitForFont(editorFont)
        let newEditorSize = editorFont.pointSize + editorSizeStep
        editorFont = NSFontManager.shared.convert(editorFont,
                                                  toSize: newEditorSize)

        let consoleSizeStep = sizeAdjustmentUnitForFont(consoleFont)
        let newConsoleSize = consoleFont.pointSize + consoleSizeStep
        consoleFont = NSFontManager.shared.convert(consoleFont,
                                                   toSize: newConsoleSize)
    }

    func decreaseFontSize() {
        let editorSizeStep = sizeAdjustmentUnitForFont(editorFont)
        let newEditorSize = editorFont.pointSize - editorSizeStep
        editorFont = NSFontManager.shared.convert(editorFont,
                                                  toSize: newEditorSize)

        let consoleSizeStep = sizeAdjustmentUnitForFont(consoleFont)
        let newConsoleSize = consoleFont.pointSize - consoleSizeStep
        consoleFont = NSFontManager.shared.convert(consoleFont,
                                                   toSize: newConsoleSize)
    }

    // As the font size increases, the step size should also increase.
    // This is a very hacky approximation of that behaviour.
    private func sizeAdjustmentUnitForFont(_ font: NSFont) -> CGFloat {
        let adjustmentFactor: CGFloat = 10.0
        return max(1, (font.pointSize / adjustmentFactor).rounded(.down))
    }
}



class PreferencesViewController: NSViewController {

    @IBOutlet weak var indentSelectButton: NSPopUpButton!
    @IBOutlet weak var tabWidthTextField: NSTextField!
    @IBOutlet weak var lineWrappingCheckButton: NSButton!
    @IBOutlet weak var autoIndentCheckButton: NSButton!
    @IBOutlet weak var tabWidthStepper: NSStepper!
    @IBOutlet weak var editorFontButton: NSButton!
    @IBOutlet weak var consoleFontButton: NSButton!

    override func viewDidLoad() {
        super.viewDidLoad()

        let formatter = NumberFormatter()
        formatter.allowsFloats = false
        formatter.minimum = MIN_TAB_WIDTH as NSNumber
        formatter.maximum = MAX_TAB_WIDTH as NSNumber
        tabWidthTextField.formatter = formatter
        tabWidthTextField.delegate = self as? NSTextFieldDelegate

        tabWidthStepper.minValue = Double(MIN_TAB_WIDTH)
        tabWidthStepper.maxValue = Double(MAX_TAB_WIDTH)

        // set initial values from saved preferences
        let useSpaces = EditorPreferences.shared.indentWithSpaces
        tabWidthStepper.integerValue = EditorPreferences.shared.tabWidth
        tabWidthTextField.integerValue = EditorPreferences.shared.tabWidth
        tabWidthTextField.isEnabled = useSpaces

        let indentMenuSelectedTag = useSpaces ? SPACES_MENU_ITEM_TAG : TAB_MENU_ITEM_TAG
        indentSelectButton.selectItem(withTag: indentMenuSelectedTag)

        autoIndentCheckButton.state = EditorPreferences.shared.autoIndentEnabled ? .on : .off
        lineWrappingCheckButton.state = EditorPreferences.shared.lineWrappingEnabled ? .on : .off

        updateFontButtonTitles()

        NotificationCenter.default.addObserver(self,
                                               selector: #selector(fontChangedNotification),
                                               name: EditorPreferences.editorFontChangedNotification,
                                               object: nil)

        NotificationCenter.default.addObserver(self,
                                               selector: #selector(fontChangedNotification),
                                               name: EditorPreferences.consoleFontChangedNotification,
                                               object: nil)
    }

    @IBAction func indentSelectAction(_ sender: NSPopUpButton) {
        let useSpaces = sender.selectedItem?.tag == SPACES_MENU_ITEM_TAG
        EditorPreferences.shared.indentWithSpaces = useSpaces
        tabWidthTextField.isEnabled = useSpaces
    }

    @IBAction func tabWidthTextFieldAction(_ sender: NSTextField) {
        tabWidthStepper.integerValue = sender.integerValue
        EditorPreferences.shared.tabWidth = sender.integerValue
    }

    @IBAction func tabWidthStepperAction(_ sender: NSStepper) {
        tabWidthTextField.integerValue = sender.integerValue
        EditorPreferences.shared.tabWidth = sender.integerValue
    }

    @IBAction func autoIndentCheckAction(_ sender: NSButton) {
        EditorPreferences.shared.autoIndentEnabled = sender.state == .on
    }

    @IBAction func lineWrappingCheckAction(_ sender: NSButton) {
        EditorPreferences.shared.lineWrappingEnabled = sender.state == .on
    }

    // frankly I just don't feel like figuring out a cleverer way to do this -_-
    private var activeFontButtonIsTheEditorOne: Bool = false

    @IBAction func editorFontButtonAction(_ sender: NSButton) {
        activeFontButtonIsTheEditorOne = true
        showFontManager()
    }

    @IBAction func consoleFontButtonAction(_ sender: NSButton) {
        activeFontButtonIsTheEditorOne = false
        showFontManager()
    }

    @objc func fontChangedNotification(_ notification: Notification) {
        updateFontButtonTitles()
    }

    private func updateFontButtonTitles() {
        let editorFont = EditorPreferences.shared.editorFont
        let consoleFont = EditorPreferences.shared.consoleFont

        editorFontButton.title = "\(editorFont.familyName ?? editorFont.fontName) \(editorFont.pointSize)"

        consoleFontButton.title = "\(consoleFont.familyName ?? consoleFont.fontName) \(consoleFont.pointSize)"
    }

    private func showFontManager() {
        NSFontManager.shared.target = self
        NSFontManager.shared.orderFrontFontPanel(nil)
    }
}

extension PreferencesViewController: NSFontChanging {
    func changeFont(_ sender: NSFontManager?) {
        if activeFontButtonIsTheEditorOne {
            if let font = sender?.convert(EditorPreferences.shared.editorFont) {
                editorFontButton.title = "\(font.familyName ?? font.fontName) \(font.pointSize)"
                EditorPreferences.shared.editorFont = font
            }
        } else {
            // it must be the other one!
            if let font = sender?.convert(EditorPreferences.shared.consoleFont) {
                consoleFontButton.title = "\(font.familyName ?? font.fontName) \(font.pointSize)"
                EditorPreferences.shared.consoleFont = font
            }
        }
    }
}
