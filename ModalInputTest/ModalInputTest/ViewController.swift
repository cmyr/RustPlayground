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

    var contents = String();

    var core: XiCoreProxy!

    enum Mode: String {
        case command, insert
    }

    var totalLines: Int = 0;

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
        if core.hasInputHandler {
            mode = .insert
        } else {
            mode = nil
        }
    }

    override func viewDidAppear() {
        self.editView.lineSource = self
        self.view.window?.makeFirstResponder(self)
    }

    func coreViewDidChange(core: XiCoreProxy, newLines: UInt32) {
        self.totalLines = Int(newLines)
        self.editView.needsDisplay = true
    }

    var contentSize: CGSize = CGSize.zero {
        didSet {
            if contentSize != oldValue {
            updateContentSize()
            }
        }
    }

    func scrollTo(_ line: Int, col: Int) {

        let y = CGFloat(line) * DefaultFont.shared.linespace + 2
        let lineText = core.getLine(UInt32(line))!
        let toMeasure = lineText.text.utf8.prefix(col)
        let x = measureStringWidth(String(toMeasure)!).width

        let rect = CGRect(origin: CGPoint(x: x, y: y),
                          size: CGSize(width: DefaultFont.shared.characterWidth(), height: DefaultFont.shared.linespace)).integral
        print("scrollTo \(rect)")
        editView.scrollToVisible(rect)
    }

    @objc func frameDidChangeNotification(_ notification: Notification) {
        core.frameChanged(newFrame: view.frame)
        updateContentSize()
    }

    func updateContentSize() {
        let cursorPadding = (DefaultFont.shared.characterWidth() * 3).rounded(.down)
        let size = CGSize(
            width: max(contentSize.width + cursorPadding, scrollView.contentSize.width),
            height: max(contentSize.height, scrollView.contentSize.height)
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
