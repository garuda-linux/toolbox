# Sharing a text as an encrypted paste: the Diagnostics report or the
# Console's output. $subject is "report" or "output".

share-link-copied = Copied the link to the clipboard.
share-delete-link-copied = Copied the delete link to the clipboard.

# The question before an upload.
share-upload-question = { $subject ->
    [report] Upload this report?
   *[output] Upload this output?
}
share-upload-where = The text below goes to bin.garudalinux.org. It is encrypted on this computer, so the server cannot read it, but anyone who has the link can. The server deletes it after one year.
share-upload-check = Check it for anything you do not want to share, such as names, addresses or serial numbers.
share-upload-text = The text that will be uploaded
share-upload-cancel = Cancel
share-upload-confirm = Upload

# The upload and its link.
share-uploading = { $subject ->
    [report] Uploading the report…
   *[output] Uploading the output…
}
share-uploaded = { $subject ->
    [report] Uploaded. Anyone who has this link can read the report:
   *[output] Uploaded. Anyone who has this link can read the output:
}
share-link = { $subject ->
    [report] Link to the uploaded report
   *[output] Link to the uploaded output
}
share-copy-link = Copy link
# The one row a place with little room shows after an upload.
share-uploaded-short = Uploaded.
share-link-actions = More link actions
share-open-link = Open in browser
share-delete-link = { $subject ->
    [report] Delete link: opening it removes the report from the server.
   *[output] Delete link: opening it removes the output from the server.
}
share-copy-delete-link = Copy delete link
share-delete-now = Delete now…
share-hide = Hide
share-delete-question = { $subject ->
    [report] Delete the report from the server?
   *[output] Delete the output from the server?
}
share-delete-detail = Nobody can open the link afterwards, you included. Your browser opens the delete link.
share-delete-cancel = Cancel
share-delete-confirm = Delete

# Why an upload failed. $detail is the system's or the server's words.
share-upload-network = The upload failed. bin.garudalinux.org could not be reached. Check the network connection.
share-upload-refused = { $subject ->
    [report] The upload failed. The paste service refused the report, for example because it is too large.
   *[output] The upload failed. The paste service refused the output, for example because it is too large.
}
share-upload-bad-answer = The upload failed. The paste service sent an answer the Toolbox does not understand.
share-upload-seal = { $subject ->
    [report] The upload failed. The report could not be encrypted.
   *[output] The upload failed. The output could not be encrypted.
}
share-upload-stopped = The upload failed because of an error in Garuda Toolbox. Report it on the forum or the issue tracker.
share-upload-detail = { $problem } ({ $detail })
share-upload-dismiss = Dismiss
