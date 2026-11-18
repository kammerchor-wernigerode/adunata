# Spec Delta

## ADDED Requirements

### Requirement: Server certificate checked against the host's trust store
On an encrypted connection, Adunata SHALL verify the certificate of the mail server against the certificate authorities in the trust store of the host it runs on. It SHALL send no mail over a connection whose certificate that store does not trust, and SHALL log the failure as one of encryption.

#### Scenario: Certificate from an authority in the store
- **WHEN** the host's trust store contains the authority that issued the mail server's certificate, whether public or added by the operator
- **THEN** Adunata delivers the mail

#### Scenario: Untrusted certificate
- **WHEN** the mail server presents a certificate that no authority in the host's trust store vouches for
- **THEN** Adunata does not send the mail and logs the failure as one of encryption

#### Scenario: Image without further settings
- **WHEN** Adunata runs in its OCI image and the operator sets no trust store
- **THEN** Adunata verifies the certificate against the bundle of certificate authorities that the image ships

#### Scenario: Image with the operator's bundle
- **WHEN** the operator of the image sets `SSL_CERT_FILE` to a bundle that contains the authority of the mail server
- **THEN** Adunata verifies against that bundle instead of the one the image ships and delivers the mail
