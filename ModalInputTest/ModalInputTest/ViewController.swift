//
//  ViewController.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-03-18.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Cocoa

class ViewController: NSViewController {

    @IBOutlet weak var editView: EditView!
    @IBOutlet weak var scrollView: NSScrollView!
    @IBOutlet weak var modeLabel: NSTextFieldCell!

    var contents = String()
    let minimumPadding: CGFloat = 2

    var core: XiCoreProxy!

    enum Mode: String {
        case command, insert, visual

        var drawBox: Bool {
            switch self {
            case .command, .visual:
                return true
            case .insert:
                return false
            }
        }
    }

    var totalLines: Int = 1;

    var parseState: String = "" {
        didSet {
            if let mode = self.mode {
                self.modeLabel.stringValue = "\(mode.rawValue.capitalized) \(parseState)"
            }
        }
    }

    var mode: Mode? = nil {
        didSet {
            if let mode = mode {
                self.modeLabel.controlView?.isHidden = false
                self.modeLabel.stringValue = mode.rawValue.capitalized(with: nil)
                self.editView.needsDisplay = true
            } else {
                self.modeLabel.controlView?.isHidden = true
            }
        }
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        NotificationCenter.default.addObserver(self,
                                               selector: #selector(frameDidChangeNotification),
                                               name: NSView.frameDidChangeNotification,
                                               object: scrollView)
        (NSApp.delegate as! AppDelegate).mainController = self
        modeLabel.font = DefaultFont.shared
        modeLabel.textColor = NSColor.lightGray
        if core.hasInputHandler {
            mode = .insert
        } else {
            mode = nil
        }
    }

    override func viewDidAppear() {
        self.editView.lineSource = self
        self.view.window?.makeFirstResponder(self)
        updateCoreFrame()
    }

    func coreViewDidChange(core: XiCoreProxy, newLines: UInt32) {
        self.totalLines = Int(newLines)
        self.editView.needsDisplay = true
    }


    /// The total size of the document, tracked in core.
    var documentSize: CGSize = CGSize.zero {
        didSet {
            if documentSize != oldValue {
            updateContentSize()
            }
        }
    }

    func scrollTo(_ line: Int, col: Int) {
        let y = CGFloat(line) * DefaultFont.shared.linespace + minimumPadding
        let lineText = core.getLine(UInt32(line))!
        let toMeasure = lineText.text.utf8.prefix(col)
        let x = measureStringWidth(String(toMeasure)!).width

        let rect = CGRect(origin: CGPoint(x: x, y: y),
                          size: CGSize(width: DefaultFont.shared.characterWidth(), height: DefaultFont.shared.linespace)).integral
        editView.scrollToVisible(rect)
    }

    @objc func frameDidChangeNotification(_ notification: Notification) {
        updateCoreFrame()
    }

    private var coreFrame = CGRect.zero {
        didSet {
            if coreFrame != oldValue {
                core.frameChanged(newFrame: coreFrame)
                updateContentSize()
            }
        }
    }

    /// Send the current frame to core. This is used for determining the visible
    /// region, and for word wrapping.
    func updateCoreFrame() {
        let docFrame = scrollView.documentVisibleRect
        let cursorPadding = DefaultFont.shared.characterWidth() + minimumPadding * 2
        let size = CGSize(width: docFrame.width - cursorPadding, height: docFrame.height)
        coreFrame = CGRect(origin: docFrame.origin, size: size)
    }

    /// Update the the size of the edit view.
    ///
    /// This is done if our frame changes (user interaction) or if the total size
    /// of the document changes (by adding or removing lines, or changing the
    /// width of the widest line.
    func updateContentSize() {
        let size = CGSize(
            width: max(documentSize.width, scrollView.contentSize.width),
            height: max(documentSize.height, scrollView.contentSize.height)
        )
        if size != editView.bounds.size {
            self.editView.frame = NSRect(origin: CGPoint.zero, size: size).integral
        }
    }

    override func doCommand(by selector: Selector) {
        let selString = NSStringFromSelector(selector)
        core.doCommand(selString)
    }

    override func insertText(_ insertString: Any) {
        core.insertText(insertString as! String)
        super.insertText(insertString)
    }

    override func keyDown(with event: NSEvent) {
        self.interpretKeyEvents([event])
    }

    @objc func paste(_ sender: AnyObject?) {
        print("paste")
        NSPasteboard
            .general
            .string(forType: .string)
            .flatMap({ core.insertText($0) })
    }

    override func selectAll(_ sender: Any?) {
        core.sendRpc(method: "selectAll:", params: [])
    }

//    @objc func undo(_ sender: AnyObject?) {
//        document.sendRpcAsync("undo", params: [])
//    }
//
//    @objc func redo(_ sender: AnyObject?) {
//        document.sendRpcAsync("redo", params: [])
//    }

//    @objc func cut(_ sender: AnyObject?) {
//        let text = xiView.cut()
//        updatePasteboard(with: text)
//    }
//
//    @objc func copy(_ sender: AnyObject?) {
//        let text = xiView.copy()
//        updatePasteboard(with: text)
//    }

    override func indent(_ sender: Any?) {
        core.sendRpc(method: "indent", params: [])
    }

    @objc func unindent(_ sender: Any?) {
        core.sendRpc(method: "outdent", params: [])
    }

    @objc func reindent(_ sender: Any?) {
        core.sendRpc(method: "reindent", params: [])
    }
}

extension ViewController: LineSource {
    func getLine(line: UInt32) -> RawLine? {
        return core?.getLine(line)
    }
}
