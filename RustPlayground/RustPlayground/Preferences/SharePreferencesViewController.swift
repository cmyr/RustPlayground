//
//  SharePreferencesViewController.swift
//  RustPlayground
//
//  Created by Colin Rofls on 2019-05-05.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Cocoa

let GITHUB_TOKEN_PATH = "https://github.com/settings/tokens"

class SharePreferencesViewController: NSViewController {

    @IBOutlet weak var githubTokenTextField: NSTextField!

    override func viewDidLoad() {
        super.viewDidLoad()
        githubTokenTextField.formatter = nil
        githubTokenTextField.delegate = self
        let token = EditorPreferences.shared.githubToken
        if token != "" {
            githubTokenTextField.stringValue = token
        }
    }

    override func viewDidAppear() {
        super.viewDidAppear()
        validateTokenTextField()
    }

    override func resignFirstResponder() -> Bool {
        return super.resignFirstResponder()
    }

    @IBAction func generateLinkAction(_ sender: NSButton) {
        guard let url = URL(string: GITHUB_TOKEN_PATH) else {
            print("conversion to url failed?")
            return
        }
        NSWorkspace.shared.open(url)
    }

    private func validateTokenTextField() {
        if textLooksLikeToken(githubTokenTextField.stringValue) {
            // deselect the text view
            self.view.window?.makeFirstResponder(self)
            githubTokenTextField.textColor = NSColor.systemGreen

            //FIXME: we should be using keychain for this, really
            EditorPreferences.shared.githubToken = githubTokenTextField.stringValue
        } else {
            githubTokenTextField.textColor = NSColor.textColor
        }
    }
}

extension SharePreferencesViewController: NSTextFieldDelegate {
    func controlTextDidChange(_ obj: Notification) {
        validateTokenTextField()
    }
}

fileprivate func textLooksLikeToken(_ maybeToken: String) -> Bool {
    return maybeToken.count == 40 && maybeToken.allSatisfy({ $0.isHexDigit })
}
