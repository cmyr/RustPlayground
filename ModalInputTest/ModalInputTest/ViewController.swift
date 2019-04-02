//
//  ViewController.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-03-18.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Cocoa

class ViewController: NSViewController {

    @IBOutlet weak var modeLabel: NSTextField!
    @IBOutlet weak var stateLabel: NSTextField!
    @IBOutlet weak var editView: EditView!

    var contents = String();

    var core: XiCoreProxy!

    enum Mode: String {
        case command, insert
    }

    enum Motion: String {
        case left, right, up, down, word, word_back, end_of_line, start_of_line
    }

    var totalLines: Int = 0;

    var parseState: String = "" {
        didSet {
            self.stateLabel.stringValue = parseState
        }
    }

    var mode: Mode = .insert {
        didSet {
            self.modeLabel.stringValue = mode.rawValue.capitalized(with: nil)
        }
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        (NSApp.delegate as! AppDelegate).mainController = self
    }

    override func viewDidAppear() {
        self.mode = .insert
        self.editView.lineSource = self

    }

    func coreViewDidChange(core: XiCoreProxy, newLines: UInt32) {
        self.totalLines = Int(newLines)
        self.editView.needsDisplay = true
    }

    override func doCommand(by selector: Selector) {
        let selString = NSStringFromSelector(selector)
        (NSApp.delegate as! AppDelegate).core.doCommand(selString)
    }

    override func insertText(_ insertString: Any) {
        (NSApp.delegate as! AppDelegate).core.insertText(insertString as! String)
        super.insertText(insertString)
    }


    override func keyDown(with event: NSEvent) {
        self.interpretKeyEvents([event])
    }

    override func flagsChanged(with event: NSEvent) {
        super.flagsChanged(with: event)
//        print(String(event.modifierFlags.rawValue, radix: 2, uppercase: true), event.keyCode)
    }


    func selectorForDeleteMotion(motion: Motion) -> Selector {
        switch motion {
        case .left:
            return #selector(NSStandardKeyBindingResponding.moveBackwardAndModifySelection(_:))
        case .right:
            return #selector(NSStandardKeyBindingResponding.moveForwardAndModifySelection(_:))
        case .word:
            return #selector(NSStandardKeyBindingResponding.moveWordForwardAndModifySelection(_:))
        case .word_back:
            return #selector(NSStandardKeyBindingResponding.moveWordBackwardAndModifySelection(_:))
        case .down:
            return #selector(NSStandardKeyBindingResponding.moveDownAndModifySelection(_:))
        case .up:
            return #selector(NSStandardKeyBindingResponding.moveUpAndModifySelection(_:))
        case .start_of_line:
            return #selector(NSStandardKeyBindingResponding.moveToBeginningOfLineAndModifySelection(_:))
        case .end_of_line:
            return #selector(NSStandardKeyBindingResponding.moveToEndOfLineAndModifySelection(_:))
        }
    }

    func selectorForMoveMotion(motion: Motion) -> Selector {
        switch motion {
        case .left:
            return #selector(NSStandardKeyBindingResponding.moveBackward(_:))
        case .right:
            return #selector(NSStandardKeyBindingResponding.moveForward(_:))
        case .word:
            return #selector(NSStandardKeyBindingResponding.moveWordForward(_:))
        case .word_back:
            return #selector(NSStandardKeyBindingResponding.moveWordBackward(_:))
        case .down:
            return #selector(NSStandardKeyBindingResponding.moveDown(_:))
        case .up:
            return #selector(NSStandardKeyBindingResponding.moveUp(_:))
        case .start_of_line:
            return #selector(NSStandardKeyBindingResponding.moveToBeginningOfLine(_:))
        case .end_of_line:
            return #selector(NSStandardKeyBindingResponding.moveToEndOfLine(_:))
        }
    }
}

extension ViewController: LineSource {
    func getLine(line: UInt32) -> RawLine? {
        return core?.getLine(line)
    }
}
