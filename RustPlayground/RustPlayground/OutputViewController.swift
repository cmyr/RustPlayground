//
//  OutputViewController.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-04-16.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Cocoa

class OutputViewController: NSViewController {
    var outputFont: NSFont {
        return EditorPreferences.shared.consoleFont
    }

    @IBOutlet var outputTextView: NSTextView!

    override func viewDidLoad() {
        super.viewDidLoad()
        outputTextView.frame = view.frame
        NotificationCenter.default.addObserver(self,
                                               selector: #selector(consoleFontChanged),
                                               name: EditorPreferences.consoleFontChangedNotification,
                                               object: nil)
    }

    @objc func consoleFontChanged(_ notification: Notification) {
        outputTextView.font = self.outputFont
        outputTextView.needsDisplay = true
    }

    func clearOutput() {
        self.outputTextView.string = ""
    }

    private func appendString(_ string: String, attributes: [NSAttributedString.Key : Any]?) {
        let EOFRange = NSRange(location: self.outputTextView.textStorage?.length ?? 0, length: 0)
        self.outputTextView.textStorage!.replaceCharacters(in: EOFRange, with: string)
        if EOFRange.location == 0 {
            self.outputTextView.font = self.outputFont
        }

        let attributes = attributes ?? [.foregroundColor: NSColor.textColor]
        let insertedRange = NSRange(location: EOFRange.location, length: string.count)
        self.outputTextView.textStorage?.addAttributes(attributes, range: insertedRange)
    }

    func printHeader(_ text: String) {
        let text = "[ \(text) ]\n"
        appendString(text, attributes: [.foregroundColor: NSColor.systemGray])
    }

    func printText(_ text: String) {
        appendString(text, attributes: nil)
    }
}

extension OutputViewController: RunnerOutputHandler {
    func printInfo(text: String) {
        // hack to give stdout time to flush before printing 'done'
        DispatchQueue.main.asyncAfter(deadline: .now() + .milliseconds(5)) {
            self.printHeader(text)
        }
    }

    func handleStdOut(text: String) {
        DispatchQueue.main.async {
            self.appendString(text, attributes: nil)
        }
    }

    func handleStdErr(text: String) {
        DispatchQueue.main.async {
            self.appendString(text, attributes: nil)
        }
    }
}
