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
    @IBOutlet weak var bottomHelperButton: NSButton!

    override func viewDidLoad() {
        super.viewDidLoad()
        githubTokenTextField.formatter = nil
        githubTokenTextField.delegate = self
        setupInitialState()
    }

    override func viewDidAppear() {
        super.viewDidAppear()
        if !githubTokenTextField.stringValue.isEmpty {
            // if we have a valid token, don't make text field the first responder
            view.window?.makeFirstResponder(self)
        }
    }

    func setupInitialState() {
        let token = EditorPreferences.shared.githubToken
        githubTokenTextField.stringValue = token
        githubTokenTextField.isEnabled = token == ""
        bottomHelperButton.target = self

        if token.isEmpty {
            bottomHelperButton.title = "Generate a token"
            bottomHelperButton.contentTintColor = NSColor.linkColor
            bottomHelperButton.action = #selector(generateLinkAction(_:))
        } else {
            bottomHelperButton.title = "Clear token"
            bottomHelperButton.contentTintColor = NSColor.systemRed
            bottomHelperButton.action = #selector(clearTokenAction(_:))
        }
    }

    @IBAction func generateLinkAction(_ sender: NSButton) {
        guard let url = URL(string: GITHUB_TOKEN_PATH) else {
            print("conversion to url failed?")
            return
        }
        NSWorkspace.shared.open(url)
    }

    @objc func clearTokenAction(_ sender: NSButton) {
        EditorPreferences.shared.githubToken = ""
        setupInitialState()
    }

    private func validateTokenTextField() {
        let maybeToken = githubTokenTextField.stringValue
        if textLooksLikeToken(maybeToken) {
            view.window?.makeFirstResponder(self)
            validateCredentialsIfPossible(maybeToken)
        }
    }

    /// If there is a valid looking token, ping github and see if it works
    /// and has the necessary authorizations.
    private func validateCredentialsIfPossible(_ token: String) {
        GithubConnection(username: "", token: token).validate {
            [weak self] (error) in
            if let error = error {
                self?.validationFailed(withError: error)
            } else {
                self?.validationSucceeded(withToken: token)
            }
        }
    }

    private func validationFailed(withError error: GithubError) {
        // not sure we should be doing much else here
        guard let window = view.window else { return }

        let alert = NSAlert(error: error)
        alert.messageText = error.localizedDescription
        alert.beginSheetModal(for: window) { [weak self] (response) in
            window.makeFirstResponder(self?.githubTokenTextField)
        }
    }

    private func validationSucceeded(withToken token: String) {
        EditorPreferences.shared.githubToken = token
        setupInitialState()
    }
}

extension SharePreferencesViewController: NSTextFieldDelegate {
    func controlTextDidChange(_ obj: Notification) {
        guard let textField = obj.object as? NSTextField else { return }

        if textField == githubTokenTextField {
            validateTokenTextField()
        }
    }
}

fileprivate func textLooksLikeToken(_ maybeToken: String) -> Bool {
    return maybeToken.count == 40 && maybeToken.allSatisfy({ $0.isHexDigit })
}
