//
//  Runner.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-04-15.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Foundation

protocol RunnerOutputHandler {
    func printInfo(text: String);
    func handleStdOut(text: String);
    func handleStdErr(text: String);
}

/// Responsible for compiling and running a snippet.
class Runner {
    //TODO: not this
    let workDirectory = URL(string: "/Users/rofls/dev/hacking/macos_rustplay_test")!
    let scriptPath: URL
    let fileName: String
    let targetName: String

    init(scriptPath: URL, fileName: String) {
        self.scriptPath = scriptPath
        self.fileName = fileName
        self.targetName = URL(string: fileName)!.deletingPathExtension().path
    }

    func compile(handler: RunnerOutputHandler) -> Bool {
        handler.printInfo(text: "Compiling")
        let task = Process()
        task.launchPath = scriptPath.path
        task.currentDirectoryPath = workDirectory.path
        //        task.arguments = ["rustc", fileName]
        let errPipe = Pipe()
        let outPipe = Pipe()

        task.standardError = errPipe
        task.standardOutput = outPipe

        outPipe.fileHandleForReading.readabilityHandler = { handle in
            let data = handle.availableData
            if let outString = String(data: data, encoding: .utf8) {
                if outString.count > 0 {
                    handler.handleStdOut(text: outString)
                }
            }
        }

        errPipe.fileHandleForReading.readabilityHandler = { handle in
            let data = handle.availableData
            if let errString = String(data: data, encoding: .utf8) {
                if errString.count > 0 {
                    handler.handleStdErr(text: errString)
                }
            }
        }

        task.launch()
        task.waitUntilExit()
        if task.terminationStatus != 0 {
            handler.printInfo(text: "Compilation failed")
            print("build failed")
        }
        return task.terminationStatus == 0
    }

    func run(handler: RunnerOutputHandler) {
        handler.printInfo(text: "Running")
        let targetPath = workDirectory.appendingPathComponent(targetName, isDirectory: false).path
        let task = Process()
        task.launchPath = targetPath
        task.currentDirectoryPath = workDirectory.path
        let errPipe = Pipe()
        let outPipe = Pipe()

        task.standardError = errPipe
        task.standardOutput = outPipe

        outPipe.fileHandleForReading.readabilityHandler = { handle in
            let data = handle.availableData
            if let outString = String(data: data, encoding: .utf8) {
                if outString.count > 0 {
                    handler.handleStdOut(text: outString)
                }
            }
        }

        errPipe.fileHandleForReading.readabilityHandler = { handle in
            let data = handle.availableData
            if let errString = String(data: data, encoding: .utf8) {
               if errString.count > 0 {
                    handler.handleStdErr(text: errString)
                }
            }
        }

        task.launch()
        task.waitUntilExit()
        handler.printInfo(text: "Done")
    }
}

func runCommand(atPath path: String, handler: RunnerOutputHandler) {
    handler.printInfo(text: "Running")

    let task = Process()
    task.launchPath = path

    let errPipe = Pipe()
    let outPipe = Pipe()

    task.standardError = errPipe
    task.standardOutput = outPipe

    outPipe.fileHandleForReading.readabilityHandler = { handle in
        let data = handle.availableData
        if let outString = String(data: data, encoding: .utf8) {
            if outString.count > 0 {
                handler.handleStdOut(text: outString)
            }
        }
    }

    errPipe.fileHandleForReading.readabilityHandler = { handle in
        let data = handle.availableData
        if let errString = String(data: data, encoding: .utf8) {
            if errString.count > 0 {
                handler.handleStdErr(text: errString)
            }
        }
    }

    task.launch()
    task.waitUntilExit()
}
